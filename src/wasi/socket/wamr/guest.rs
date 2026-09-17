//! Calls from a guest built against WAMR's socket library.

use super::layout::*;
use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::structure::module::WamrSockOpt;
use crate::wasi::passthrough::PassthroughWasiImpl;
use crate::wasi::socket::{
    decode, iovecs, param_i32, param_i64, read_array, read_cstr, read_struct, write_bytes,
    write_struct, Backend, Host, OptValue, RESOLVE_LIMIT,
};
use crate::wasi::{WasiError, WasiResult};

/// `sock_open(pool_fd, family, type, fd_out)`; the pool fd is a stub.
pub(crate) fn open(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let family = decode(&FAMILY_CODES, param_i32(params, 1)?)?;
    let ty = decode(&TYPE_CODES, param_i32(params, 2)?)?;
    let fd = Host::open(family, ty)?;
    write_bytes(memory, param_i32(params, 3)?, &fd.to_le_bytes())
}

/// `sock_bind(fd, addr)`
pub(crate) fn bind(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = decode_addr(&read_array(memory, param_i32(params, 1)?)?)?;
    Host::bind(param_i32(params, 0)?, &addr)
}

/// `sock_connect(fd, addr)`
pub(crate) fn connect(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = decode_addr(&read_array(memory, param_i32(params, 1)?)?)?;
    Host::connect(param_i32(params, 0)?, &addr)
}

/// `sock_addr_local(fd, addr_out)`
pub(crate) fn addr_local(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = Host::local_addr(param_i32(params, 0)?)?;
    write_bytes(memory, param_i32(params, 1)?, &encode_addr(&addr))
}

/// `sock_addr_remote(fd, addr_out)`
pub(crate) fn addr_remote(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let addr = Host::peer_addr(param_i32(params, 0)?)?;
    write_bytes(memory, param_i32(params, 1)?, &encode_addr(&addr))
}

/// `sock_recv_from(fd, iovs, iovs_len, flags, addr_out, len_out)`
pub(crate) fn recv_from(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let iovecs = iovecs(memory, params)?;
    let (len, from) =
        Host::recv_from(param_i32(params, 0)?, &iovecs, param_i32(params, 3)? as u32)?;
    write_bytes(memory, param_i32(params, 4)?, &encode_addr(&from))?;
    write_bytes(memory, param_i32(params, 5)?, &len.to_le_bytes())
}

/// `sock_send_to(fd, iovs, iovs_len, flags, addr, len_out)`
pub(crate) fn send_to(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let to = decode_addr(&read_array(memory, param_i32(params, 4)?)?)?;
    let iovecs = iovecs(memory, params)?;
    let len = Host::send_to(
        param_i32(params, 0)?,
        &iovecs,
        param_i32(params, 3)? as u32,
        &to,
    )?;
    write_bytes(memory, param_i32(params, 5)?, &len.to_le_bytes())
}

pub(crate) fn close(wasi: &PassthroughWasiImpl, params: &[Val]) -> WasiResult<()> {
    match wasi.fd_close(param_i32(params, 0)?)? {
        0 => Ok(()),
        errno => Err(WasiError::from_errno(errno as u16)),
    }
}

/// `sock_addr_resolve(host, service, hints, out, out_count, total_out)`: C
/// strings in; up to `out_count` results out, and how many there were.
pub(crate) fn addr_resolve(memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let host = read_cstr(memory, param_i32(params, 0)?)?;
    let service = read_cstr(memory, param_i32(params, 1)?)?;
    let hints: Hints = read_struct(memory, param_i32(params, 2)?)?;
    let (family, ty) = hints.decode()?;
    let out = param_i32(params, 3)?;
    let count = (param_i32(params, 4)? as u32 as usize).min(RESOLVE_LIMIT);
    let (found, total) = Host::resolve(&host, &service, family, ty, count)?;
    for (i, item) in found.iter().enumerate() {
        let at = out.wrapping_add((i * std::mem::size_of::<AddrInfo>()) as i32);
        write_struct(memory, at, &AddrInfo::new(item))?;
    }
    write_bytes(memory, param_i32(params, 5)?, &(total as u32).to_le_bytes())
}

/// `sock_set_*(fd, value)`: a bool or size as i32, a timeout as i64
/// microseconds, linger as `(fd, on, secs)`.
pub(crate) fn set_opt(opt: WamrSockOpt, params: &[Val]) -> WasiResult<()> {
    let opt = common_opt(opt)?;
    let value = match opt.blank() {
        OptValue::Bool(_) => OptValue::Bool(param_i32(params, 1)? != 0),
        OptValue::Size(_) => OptValue::Size(param_i32(params, 1)? as u32),
        OptValue::Timeout(_) => OptValue::Timeout(param_i64(params, 1)? as u64),
        OptValue::Linger { .. } => OptValue::Linger {
            on: param_i32(params, 1)? != 0,
            secs: param_i32(params, 2)?,
        },
    };
    Host::set_opt(param_i32(params, 0)?, opt, value)
}

/// `sock_get_*(fd, out)`: a bool as one byte, a size as u32, a timeout as
/// u64, linger as `(fd, on_out, secs_out)`.
pub(crate) fn get_opt(opt: WamrSockOpt, memory: &MemAddr, params: &[Val]) -> WasiResult<()> {
    let out = param_i32(params, 1)?;
    match Host::get_opt(param_i32(params, 0)?, common_opt(opt)?)? {
        OptValue::Bool(on) => write_bytes(memory, out, &[on as u8]),
        OptValue::Size(size) => write_bytes(memory, out, &size.to_le_bytes()),
        OptValue::Timeout(us) => write_bytes(memory, out, &us.to_le_bytes()),
        OptValue::Linger { on, secs } => {
            write_bytes(memory, out, &[on as u8])?;
            write_bytes(memory, param_i32(params, 2)?, &secs.to_le_bytes())
        }
    }
}
