//! Chiwawa on WasmEdge: the extension called through WasmEdge's own imports.
//! The V2 names are imported explicitly, since a plain name whose signature
//! did not change resolves to V1, which rejects the 128-byte address form.

use super::layout::*;
use crate::wasi::passthrough::WasiIovec;
use crate::wasi::socket::{
    decode, encode, AddressFamily, Backend, OptValue, Resolved, SockOpt, SocketType,
};
use crate::wasi::{WasiError, WasiResult};
use std::net::{IpAddr, SocketAddr};

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
    fn sock_getaddrinfo(
        node: *const u8,
        node_len: u32,
        service: *const u8,
        service_len: u32,
        hints: *const AddrInfo,
        res: *mut u32,
        max: u32,
        res_len: *mut u32,
    ) -> u16;
    fn sock_setsockopt(fd: i32, level: i32, name: i32, value: *const u8, len: u32) -> u16;
    fn sock_getsockopt(fd: i32, level: i32, name: i32, value: *mut u8, len: *mut u32) -> u16;
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
            buf: self.bytes.as_mut_ptr() as u32,
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
    fn resolve(
        node: &str,
        service: &str,
        family: AddressFamily,
        ty: SocketType,
        max: usize,
    ) -> WasiResult<(Vec<Resolved>, usize)> {
        let mut data = vec![[0u8; SA_DATA_MAX]; max];
        let mut names = vec![0u8; max];
        let mut sockaddrs: Vec<SockAddr> = data
            .iter_mut()
            .map(|d| SockAddr {
                family: 0,
                _pad: [0; 3],
                data_len: SA_DATA_MAX as u32,
                data: d.as_mut_ptr() as u32,
            })
            .collect();
        let blank = AddrInfo {
            flags: 0,
            family: 0,
            socktype: 0,
            protocol: 0,
            _pad: [0; 3],
            addrlen: 0,
            addr: 0,
            canonname: 0,
            canonname_len: 0,
            next: 0,
        };
        let mut infos: Vec<AddrInfo> = sockaddrs
            .iter_mut()
            .zip(names.iter_mut())
            .map(|(addr, name)| AddrInfo {
                addr: addr as *mut SockAddr as u32,
                canonname: name as *mut u8 as u32,
                ..blank
            })
            .collect();
        for i in 1..max {
            let next = &mut infos[i] as *mut AddrInfo as u32;
            infos[i - 1].next = next;
        }
        let hints = AddrInfo {
            family: encode(&FAMILY_CODES, family) as u8,
            socktype: encode(&TYPE_CODES, ty) as u8,
            ..blank
        };
        let mut res = infos.as_mut_ptr() as u32;
        let mut res_len = 0u32;
        check(unsafe {
            sock_getaddrinfo(
                node.as_ptr(),
                node.len() as u32,
                service.as_ptr(),
                service.len() as u32,
                &hints,
                &mut res,
                max as u32,
                &mut res_len,
            )
        })?;
        let found = (0..res_len as usize)
            .map(|i| {
                Ok(Resolved {
                    addr: decode_sa_data(sockaddrs[i].family, &data[i])?,
                    ty: decode(&TYPE_CODES, infos[i].socktype as i32)?,
                })
            })
            .collect::<WasiResult<Vec<_>>>()?;
        let total = found.len();
        Ok((found, total))
    }
    fn set_opt(fd: i32, opt: SockOpt, value: OptValue) -> WasiResult<()> {
        let (bytes, len) = encode_opt(value);
        let name = encode(&OPT_CODES, opt);
        check(unsafe { sock_setsockopt(fd, LEVEL_SOCKET, name, bytes.as_ptr(), len as u32) })
    }

    fn get_opt(fd: i32, opt: SockOpt) -> WasiResult<OptValue> {
        let mut bytes = [0u8; OPT_VALUE_MAX];
        let mut len = OPT_VALUE_MAX as u32;
        let name = encode(&OPT_CODES, opt);
        check(unsafe { sock_getsockopt(fd, LEVEL_SOCKET, name, bytes.as_mut_ptr(), &mut len) })?;
        decode_opt(opt, &bytes[..len as usize])
    }
}
