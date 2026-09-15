//! Chiwawa on WAMR: the extension called through `iwasm`'s own imports.

use super::layout::*;
use crate::wasi::passthrough::WasiIovec;
use crate::wasi::socket::{encode, AddressFamily, Backend, Resolved, SocketType};
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
}
