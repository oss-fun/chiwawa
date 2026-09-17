//! Chiwawa on WAMR: the extension called through `iwasm`'s own imports.

use super::layout::*;
use crate::wasi::passthrough::WasiIovec;
use crate::wasi::socket::{
    encode, AddressFamily, Backend, OptValue, Resolved, SockOpt, SocketType,
};
use crate::wasi::{WasiError, WasiResult};
use std::ffi::CString;
use std::net::SocketAddr;

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sock_open(pool_fd: i32, family: i32, ty: i32, fd_out: *mut i32) -> u16;
    fn sock_bind(fd: i32, addr: *const u8) -> u16;
    fn sock_connect(fd: i32, addr: *const u8) -> u16;
    fn sock_listen(fd: i32, backlog: i32) -> u16;
    fn sock_addr_local(fd: i32, addr_out: *mut u8) -> u16;
    fn sock_addr_remote(fd: i32, addr_out: *mut u8) -> u16;
    fn sock_recv_from(
        fd: i32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        flags: u32,
        addr_out: *mut u8,
        len_out: *mut u32,
    ) -> u16;
    fn sock_addr_resolve(
        host: *const u8,
        service: *const u8,
        hints: *const u8,
        out: *mut u8,
        out_count: u32,
        total_out: *mut u32,
    ) -> u16;
    fn sock_set_reuse_addr(fd: i32, on: i32) -> u16;
    fn sock_get_reuse_addr(fd: i32, on_out: *mut u8) -> u16;
    fn sock_set_keep_alive(fd: i32, on: i32) -> u16;
    fn sock_get_keep_alive(fd: i32, on_out: *mut u8) -> u16;
    fn sock_set_broadcast(fd: i32, on: i32) -> u16;
    fn sock_get_broadcast(fd: i32, on_out: *mut u8) -> u16;
    fn sock_set_recv_buf_size(fd: i32, size: u32) -> u16;
    fn sock_get_recv_buf_size(fd: i32, size_out: *mut u64) -> u16;
    fn sock_set_send_buf_size(fd: i32, size: u32) -> u16;
    fn sock_get_send_buf_size(fd: i32, size_out: *mut u64) -> u16;
    fn sock_set_recv_timeout(fd: i32, us: u64) -> u16;
    fn sock_get_recv_timeout(fd: i32, us_out: *mut u64) -> u16;
    fn sock_set_send_timeout(fd: i32, us: u64) -> u16;
    fn sock_get_send_timeout(fd: i32, us_out: *mut u64) -> u16;
    fn sock_set_linger(fd: i32, on: i32, secs: i32) -> u16;
    fn sock_get_linger(fd: i32, on_out: *mut u8, secs_out: *mut i32) -> u16;
    fn sock_send_to(
        fd: i32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        flags: u32,
        addr: *const u8,
        len_out: *mut u32,
    ) -> u16;
}

fn check(errno: u16) -> WasiResult<()> {
    if errno == 0 {
        Ok(())
    } else {
        Err(WasiError::from_errno(errno))
    }
}

pub(crate) struct Wamr;

impl Backend for Wamr {
    fn open(family: AddressFamily, ty: SocketType) -> WasiResult<i32> {
        let mut fd = 0;
        let (family, ty) = (encode(&FAMILY_CODES, family), encode(&TYPE_CODES, ty));
        check(unsafe { sock_open(-1, family, ty, &mut fd) })?;
        Ok(fd)
    }
    fn bind(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        check(unsafe { sock_bind(fd, encode_addr(addr).as_ptr()) })
    }
    fn connect(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        check(unsafe { sock_connect(fd, encode_addr(addr).as_ptr()) })
    }
    fn listen(fd: i32, backlog: i32) -> WasiResult<()> {
        check(unsafe { sock_listen(fd, backlog) })
    }
    fn local_addr(fd: i32) -> WasiResult<SocketAddr> {
        let mut bytes = [0u8; ADDR_SIZE];
        check(unsafe { sock_addr_local(fd, bytes.as_mut_ptr()) })?;
        decode_addr(&bytes)
    }
    fn peer_addr(fd: i32) -> WasiResult<SocketAddr> {
        let mut bytes = [0u8; ADDR_SIZE];
        check(unsafe { sock_addr_remote(fd, bytes.as_mut_ptr()) })?;
        decode_addr(&bytes)
    }
    fn recv_from(fd: i32, iovs: &[WasiIovec], flags: u32) -> WasiResult<(u32, SocketAddr)> {
        let mut bytes = [0u8; ADDR_SIZE];
        let mut len = 0;
        check(unsafe {
            sock_recv_from(
                fd,
                iovs.as_ptr(),
                iovs.len() as u32,
                flags,
                bytes.as_mut_ptr(),
                &mut len,
            )
        })?;
        Ok((len, decode_addr(&bytes)?))
    }
    fn send_to(fd: i32, iovs: &[WasiIovec], flags: u32, to: &SocketAddr) -> WasiResult<u32> {
        let mut len = 0;
        check(unsafe {
            sock_send_to(
                fd,
                iovs.as_ptr(),
                iovs.len() as u32,
                flags,
                encode_addr(to).as_ptr(),
                &mut len,
            )
        })?;
        Ok(len)
    }
    fn resolve(
        node: &str,
        service: &str,
        family: AddressFamily,
        ty: SocketType,
        max: usize,
    ) -> WasiResult<(Vec<Resolved>, usize)> {
        let node = CString::new(node).map_err(|_| WasiError::Inval)?;
        let service = CString::new(service).map_err(|_| WasiError::Inval)?;
        let hints = Hints::new(family, ty);
        let mut out = vec![
            AddrInfo {
                addr: [0; ADDR_SIZE],
                ty: 0,
            };
            max
        ];
        let mut total = 0u32;
        check(unsafe {
            sock_addr_resolve(
                node.as_ptr() as *const u8,
                service.as_ptr() as *const u8,
                &hints as *const Hints as *const u8,
                out.as_mut_ptr() as *mut u8,
                max as u32,
                &mut total,
            )
        })?;
        let found = out[..(total as usize).min(max)]
            .iter()
            .map(AddrInfo::decode)
            .collect::<WasiResult<Vec<_>>>()?;
        Ok((found, total as usize))
    }
    fn set_opt(fd: i32, opt: SockOpt, value: OptValue) -> WasiResult<()> {
        check(unsafe {
            match (opt, value) {
                (SockOpt::ReuseAddr, OptValue::Bool(on)) => sock_set_reuse_addr(fd, on as i32),
                (SockOpt::KeepAlive, OptValue::Bool(on)) => sock_set_keep_alive(fd, on as i32),
                (SockOpt::Broadcast, OptValue::Bool(on)) => sock_set_broadcast(fd, on as i32),
                (SockOpt::RecvBufSize, OptValue::Size(size)) => sock_set_recv_buf_size(fd, size),
                (SockOpt::SendBufSize, OptValue::Size(size)) => sock_set_send_buf_size(fd, size),
                (SockOpt::RecvTimeout, OptValue::Timeout(us)) => sock_set_recv_timeout(fd, us),
                (SockOpt::SendTimeout, OptValue::Timeout(us)) => sock_set_send_timeout(fd, us),
                (SockOpt::Linger, OptValue::Linger { on, secs }) => {
                    sock_set_linger(fd, on as i32, secs)
                }
                _ => return Err(WasiError::Inval),
            }
        })
    }

    fn get_opt(fd: i32, opt: SockOpt) -> WasiResult<OptValue> {
        let (mut on, mut wide, mut secs) = (0u8, 0u64, 0i32);
        check(unsafe {
            match opt {
                SockOpt::ReuseAddr => sock_get_reuse_addr(fd, &mut on),
                SockOpt::KeepAlive => sock_get_keep_alive(fd, &mut on),
                SockOpt::Broadcast => sock_get_broadcast(fd, &mut on),
                SockOpt::RecvBufSize => sock_get_recv_buf_size(fd, &mut wide),
                SockOpt::SendBufSize => sock_get_send_buf_size(fd, &mut wide),
                SockOpt::RecvTimeout => sock_get_recv_timeout(fd, &mut wide),
                SockOpt::SendTimeout => sock_get_send_timeout(fd, &mut wide),
                SockOpt::Linger => sock_get_linger(fd, &mut on, &mut secs),
            }
        })?;
        Ok(match opt.blank() {
            OptValue::Bool(_) => OptValue::Bool(on != 0),
            OptValue::Size(_) => OptValue::Size(wide as u32),
            OptValue::Timeout(_) => OptValue::Timeout(wide),
            OptValue::Linger { .. } => OptValue::Linger { on: on != 0, secs },
        })
    }
}
