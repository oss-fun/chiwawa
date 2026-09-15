//! Chiwawa on WasmEdge: the extension called through WasmEdge's own imports.
//! The V2 names are imported explicitly, since a plain name whose signature
//! did not change resolves to V1, which rejects the 128-byte address form.

use super::layout::*;
use crate::wasi::passthrough::WasiIovec;
use crate::wasi::socket::{AddressFamily, Backend, SocketType};
use crate::wasi::{WasiError, WasiResult};
use std::net::{IpAddr, SocketAddr};

/// `__wasi_address_t`
#[repr(C)]
struct WasiAddress {
    buf: *mut u8,
    buf_len: u32,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn sock_open(family: i32, ty: i32, fd_out: *mut i32) -> u16;
    #[link_name = "sock_bind_v2"]
    fn sock_bind(fd: i32, addr: *const WasiAddress, port: u32) -> u16;
    #[link_name = "sock_connect_v2"]
    fn sock_connect(fd: i32, addr: *const WasiAddress, port: u32) -> u16;
    fn sock_listen(fd: i32, backlog: i32) -> u16;
    #[link_name = "sock_getlocaladdr_v2"]
    fn sock_getlocaladdr(fd: i32, addr: *const WasiAddress, port_out: *mut u32) -> u16;
    #[link_name = "sock_getpeeraddr_v2"]
    fn sock_getpeeraddr(fd: i32, addr: *const WasiAddress, port_out: *mut u32) -> u16;
    #[link_name = "sock_recv_from_v2"]
    fn sock_recv_from(
        fd: i32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        addr: *const WasiAddress,
        flags: u32,
        port_out: *mut u16,
        len_out: *mut u32,
        flags_out: *mut u16,
    ) -> u16;
    #[link_name = "sock_send_to_v2"]
    fn sock_send_to(
        fd: i32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        addr: *const WasiAddress,
        port: i32,
        flags: u32,
        len_out: *mut u32,
    ) -> u16;
}

fn encode_storage(ip: &IpAddr) -> [u8; STORAGE_SIZE] {
    let mut buf = [0u8; STORAGE_SIZE];
    encode_ip(&mut buf, ip).unwrap();
    buf
}

fn check(errno: u16) -> WasiResult<()> {
    if errno == 0 {
        Ok(())
    } else {
        Err(WasiError::from_errno(errno))
    }
}

/// WasmEdge accepts only a concrete family and type.
fn family_code(family: AddressFamily) -> WasiResult<i32> {
    match family {
        AddressFamily::Inet4 => Ok(1),
        AddressFamily::Inet6 => Ok(2),
        AddressFamily::Unspec => Err(WasiError::AfNoSupport),
    }
}

fn type_code(ty: SocketType) -> WasiResult<i32> {
    match ty {
        SocketType::Datagram => Ok(1),
        SocketType::Stream => Ok(2),
        SocketType::Any => Err(WasiError::Inval),
    }
}

/// A 128-byte address buffer and the `__wasi_address_t` that points at it.
struct Storage {
    bytes: [u8; STORAGE_SIZE],
}

impl Storage {
    fn of(addr: &SocketAddr) -> Self {
        Self {
            bytes: encode_storage(&addr.ip()),
        }
    }
    fn empty() -> Self {
        Self {
            bytes: [0; STORAGE_SIZE],
        }
    }
    fn address(&mut self) -> WasiAddress {
        WasiAddress {
            buf: self.bytes.as_mut_ptr(),
            buf_len: STORAGE_SIZE as u32,
        }
    }
    fn to_socket_addr(&self, port: u16) -> WasiResult<SocketAddr> {
        decode_addr(&self.bytes, port)
    }
}

pub(crate) struct WasmEdge;

impl Backend for WasmEdge {
    fn open(family: AddressFamily, ty: SocketType) -> WasiResult<i32> {
        let mut fd = 0;
        check(unsafe { sock_open(family_code(family)?, type_code(ty)?, &mut fd) })?;
        Ok(fd)
    }
    fn bind(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        let mut storage = Storage::of(addr);
        check(unsafe { sock_bind(fd, &storage.address(), addr.port() as u32) })
    }
    fn connect(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        let mut storage = Storage::of(addr);
        check(unsafe { sock_connect(fd, &storage.address(), addr.port() as u32) })
    }
    fn listen(fd: i32, backlog: i32) -> WasiResult<()> {
        check(unsafe { sock_listen(fd, backlog) })
    }
    fn local_addr(fd: i32) -> WasiResult<SocketAddr> {
        let mut storage = Storage::empty();
        let mut port = 0u32;
        check(unsafe { sock_getlocaladdr(fd, &storage.address(), &mut port) })?;
        storage.to_socket_addr(port as u16)
    }
    fn peer_addr(fd: i32) -> WasiResult<SocketAddr> {
        let mut storage = Storage::empty();
        let mut port = 0u32;
        check(unsafe { sock_getpeeraddr(fd, &storage.address(), &mut port) })?;
        storage.to_socket_addr(port as u16)
    }
    fn recv_from(fd: i32, iovs: &[WasiIovec], flags: u32) -> WasiResult<(u32, SocketAddr)> {
        let mut storage = Storage::empty();
        let (mut port, mut len, mut out_flags) = (0u16, 0u32, 0u16);
        check(unsafe {
            sock_recv_from(
                fd,
                iovs.as_ptr(),
                iovs.len() as u32,
                &storage.address(),
                flags,
                &mut port,
                &mut len,
                &mut out_flags,
            )
        })?;
        Ok((len, storage.to_socket_addr(port)?))
    }
    fn send_to(fd: i32, iovs: &[WasiIovec], flags: u32, to: &SocketAddr) -> WasiResult<u32> {
        let mut storage = Storage::of(to);
        let mut len = 0u32;
        check(unsafe {
            sock_send_to(
                fd,
                iovs.as_ptr(),
                iovs.len() as u32,
                &storage.address(),
                to.port() as i32,
                flags,
                &mut len,
            )
        })?;
        Ok(len)
    }
}
