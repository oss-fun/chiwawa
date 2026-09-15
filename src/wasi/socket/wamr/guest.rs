//! Calls from a guest built against WAMR's socket library.

use super::layout::*;
use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::wasi::passthrough::PassthroughWasiImpl;
use crate::wasi::socket::{iovecs, param_i32, read_array, write_bytes, Backend, Host};
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
