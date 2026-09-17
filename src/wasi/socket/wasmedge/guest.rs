//! Calls from a guest built against WasmEdge's socket library.

use super::layout::*;
use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::wasi::passthrough::PassthroughWasiImpl;
use crate::wasi::socket::{
    decode, encode, iovecs, param_i32, read_array, read_bytes, read_str, read_struct, write_bytes,
    write_struct, Backend, Host, RESOLVE_LIMIT,
};
use crate::wasi::{WasiError, WasiResult};
use std::net::SocketAddr;

/// The guest's `__wasi_address_t` at `ptr`: where its buffer is, how long.
fn address_buffer(memory: &MemAddr, ptr: i32) -> WasiResult<(i32, usize)> {
    let address: WasiAddress = read_struct(memory, ptr)?;
    Ok((address.buf as i32, address.buf_len as usize))
}

fn read_addr(memory: &MemAddr, ptr: i32, port: i32) -> WasiResult<SocketAddr> {
    let (buf, len) = address_buffer(memory, ptr)?;
    decode_addr(&read_bytes(memory, buf, len)?, port as u16)
}

/// Writes `addr`'s IP into the guest's buffer.
fn write_addr(memory: &MemAddr, ptr: i32, addr: &SocketAddr) -> WasiResult<()> {
    let (buf, len) = address_buffer(memory, ptr)?;
    let mut bytes = read_bytes(memory, buf, len)?;
    encode_ip(&mut bytes, &addr.ip())?;
    write_bytes(memory, buf, &bytes)
}

/// `sock_open(family, type, fd_out)`
pub(crate) fn open(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let family = decode(&FAMILY_CODES, param_i32(params, 0)?)?;
    let ty = decode(&TYPE_CODES, param_i32(params, 1)?)?;
    let fd = Host::open(family, ty)?;
    write_bytes(memory, param_i32(params, 2)?, &fd.to_le_bytes())
}

/// `sock_bind(fd, addr, port)`
pub(crate) fn bind(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = read_addr(memory, param_i32(params, 1)?, param_i32(params, 2)?)?;
    Host::bind(param_i32(params, 0)?, &addr)
}

/// `sock_connect(fd, addr, port)`
pub(crate) fn connect(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = read_addr(memory, param_i32(params, 1)?, param_i32(params, 2)?)?;
    Host::connect(param_i32(params, 0)?, &addr)
}

/// V1 `sock_accept(fd, fd_out)`: the standard accept with no flags, so the passthrough serves it.
pub(crate) fn accept_v1(
    wasi: &PassthroughWasiImpl,
    memory: &MemAddr,
    params: &[Val],
) -> WasiResult<()> {
    let fd = param_i32(params, 0)? as u32;
    let fd_out = param_i32(params, 1)? as u32;
    match wasi.sock_accept(memory, fd, 0, fd_out)? {
        0 => Ok(()),
        errno => Err(WasiError::from_errno(errno as u16)),
    }
}

/// V1 `sock_recv_from(fd, iovs, iovs_len, addr, flags, len_out, flags_out)`
pub(crate) fn recv_from_v1(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let iovecs = iovecs(memory, params)?;
    let (len, from) =
        Host::recv_from(param_i32(params, 0)?, &iovecs, param_i32(params, 4)? as u32)?;
    write_addr(memory, param_i32(params, 3)?, &from)?;
    write_bytes(memory, param_i32(params, 5)?, &len.to_le_bytes())?;
    write_bytes(memory, param_i32(params, 6)?, &0u16.to_le_bytes())
}

/// V2 `sock_recv_from(fd, iovs, iovs_len, addr, flags, port_out, len_out, flags_out)`; the port is a u16.
pub(crate) fn recv_from_v2(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let iovecs = iovecs(memory, params)?;
    let (len, from) =
        Host::recv_from(param_i32(params, 0)?, &iovecs, param_i32(params, 4)? as u32)?;
    write_addr(memory, param_i32(params, 3)?, &from)?;
    write_bytes(memory, param_i32(params, 5)?, &from.port().to_le_bytes())?;
    write_bytes(memory, param_i32(params, 6)?, &len.to_le_bytes())?;
    write_bytes(memory, param_i32(params, 7)?, &0u16.to_le_bytes())
}

/// `sock_send_to(fd, iovs, iovs_len, addr, port, flags, len_out)`
pub(crate) fn send_to(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let to = read_addr(memory, param_i32(params, 3)?, param_i32(params, 4)?)?;
    let iovecs = iovecs(memory, params)?;
    let len = Host::send_to(
        param_i32(params, 0)?,
        &iovecs,
        param_i32(params, 5)? as u32,
        &to,
    )?;
    write_bytes(memory, param_i32(params, 6)?, &len.to_le_bytes())
}

/// V1 `(fd, addr, family_out, port_out)`: the family as 4 or 6, the port as a u32.
fn addr_v1(memory: &MemAddr, params: &[Val], addr: SocketAddr) -> WasiResult<()> {
    write_addr(memory, param_i32(params, 1)?, &addr)?;
    write_bytes(
        memory,
        param_i32(params, 2)?,
        &v1_family_code(&addr.ip()).to_le_bytes(),
    )?;
    write_bytes(
        memory,
        param_i32(params, 3)?,
        &(addr.port() as u32).to_le_bytes(),
    )
}

/// V2 `(fd, addr, port_out)`: a 128-byte buffer, the port as a u32.
fn addr_v2(memory: &MemAddr, params: &[Val], addr: SocketAddr) -> WasiResult<()> {
    write_addr(memory, param_i32(params, 1)?, &addr)?;
    write_bytes(
        memory,
        param_i32(params, 2)?,
        &(addr.port() as u32).to_le_bytes(),
    )
}

pub(crate) fn local_addr_v1(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    addr_v1(memory, params, Host::local_addr(param_i32(params, 0)?)?)
}

pub(crate) fn local_addr_v2(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    addr_v2(memory, params, Host::local_addr(param_i32(params, 0)?)?)
}

pub(crate) fn peer_addr_v1(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    addr_v1(memory, params, Host::peer_addr(param_i32(params, 0)?)?)
}

pub(crate) fn peer_addr_v2(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    addr_v2(memory, params, Host::peer_addr(param_i32(params, 0)?)?)
}

/// `sock_getaddrinfo(node, node_len, service, service_len, hints, res,
/// max_len, res_len)`: fills the guest's chain of `max_len` addrinfo entries.
pub(crate) fn getaddrinfo(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let max = param_i32(params, 6)? as u32 as usize;
    if max == 0 || max > RESOLVE_LIMIT {
        return Err(WasiError::AiMemory);
    }
    let node = read_str(
        memory,
        param_i32(params, 0)?,
        param_i32(params, 1)? as u32 as usize,
    )?;
    let service = read_str(
        memory,
        param_i32(params, 2)?,
        param_i32(params, 3)? as u32 as usize,
    )?;
    if node.is_empty() && service.is_empty() {
        return Err(WasiError::AiNoName);
    }
    let hints: AddrInfo = read_struct(memory, param_i32(params, 4)?)?;
    let family = decode(&FAMILY_CODES, hints.family as i32)?;
    let ty = decode(&TYPE_CODES, hints.socktype as i32)?;

    let mut entries = Vec::with_capacity(max);
    let mut at = i32::from_le_bytes(read_array(memory, param_i32(params, 5)?)?);
    for _ in 0..max {
        let entry: AddrInfo = read_struct(memory, at)?;
        entries.push((at, entry));
        at = entry.next as i32;
    }

    let (found, _) = Host::resolve(&node, &service, family, ty, max)?;
    for (item, (at, entry)) in found.iter().zip(&entries) {
        let (family, data, len) = sa_data(&item.addr);
        let info = AddrInfo {
            flags: 0,
            family,
            socktype: encode(&TYPE_CODES, item.ty) as u8,
            protocol: protocol_code(item.ty),
            addrlen: 2 + len as u32,
            canonname_len: 0,
            ..*entry
        };
        write_struct(memory, *at, &info)?;
        let sockaddr: SockAddr = read_struct(memory, entry.addr as i32)?;
        if len > sockaddr.data_len as usize {
            return Err(WasiError::Fault);
        }
        write_struct(memory, entry.addr as i32, &SockAddr { family, ..sockaddr })?;
        write_bytes(memory, sockaddr.data as i32, &data[..len])?;
    }
    write_bytes(
        memory,
        param_i32(params, 7)?,
        &(found.len() as u32).to_le_bytes(),
    )
}

/// `sock_setsockopt(fd, level, name, value, value_len)`
pub(crate) fn setsockopt(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let opt = decode_opt_name(param_i32(params, 1)?, param_i32(params, 2)?)?;
    let len = param_i32(params, 4)? as u32 as usize;
    let value = decode_opt(opt, &read_bytes(memory, param_i32(params, 3)?, len)?)?;
    Host::set_opt(param_i32(params, 0)?, opt, value)
}

/// `sock_getsockopt(fd, level, name, value_out, value_len_inout)`
pub(crate) fn getsockopt(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let opt = decode_opt_name(param_i32(params, 1)?, param_i32(params, 2)?)?;
    let len_at = param_i32(params, 4)?;
    let room = u32::from_le_bytes(read_array(memory, len_at)?) as usize;
    let (bytes, len) = encode_opt(Host::get_opt(param_i32(params, 0)?, opt)?);
    if len > room {
        return Err(WasiError::Inval);
    }
    write_bytes(memory, param_i32(params, 3)?, &bytes[..len])?;
    write_bytes(memory, len_at, &(len as u32).to_le_bytes())
}
