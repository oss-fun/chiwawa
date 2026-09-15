//! Calls from a guest built against WasmEdge's socket library.

use super::layout::*;
use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::wasi::passthrough::PassthroughWasiImpl;
use crate::wasi::socket::{iovecs, param_i32, read_array, read_bytes, write_bytes, Backend, Host};
use crate::wasi::{WasiError, WasiResult};
use std::net::SocketAddr;

/// The guest's `__wasi_address_t` at `ptr`: where its buffer is, how long.
fn address_buffer(memory: &MemAddr, ptr: i32) -> WasiResult<(i32, usize)> {
    let bytes: [u8; 8] = read_array(memory, ptr)?;
    let buf = i32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    Ok((buf, len as usize))
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
