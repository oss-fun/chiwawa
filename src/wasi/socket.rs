//! Socket extensions of WAMR and WasmEdge.
//!
//! A guest may use either extension. The host Chiwawa itself calls is chosen
//! at build time by the `socket-wamr` or `socket-wasmedge` feature.

#[cfg(all(feature = "socket-wamr", feature = "socket-wasmedge", not(docsrs)))]
compile_error!("socket-wamr and socket-wasmedge select different hosts; enable one");

pub mod wamr;
pub mod wasmedge;

use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::structure::module::SocketExt;
use crate::wasi::passthrough::{collect_iovecs, guest_range, PassthroughWasiImpl, WasiIovec};
use crate::wasi::{WasiError, WasiResult};
use std::net::SocketAddr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AddressFamily {
    Inet4,
    Inet6,
    Unspec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SocketType {
    Stream,
    Datagram,
    Any,
}

/// One name-resolution result.
pub(crate) struct Resolved {
    pub addr: SocketAddr,
    pub ty: SocketType,
}

/// The most results a guest may ask a resolution for.
pub(crate) const RESOLVE_LIMIT: usize = 32;

/// A code from an ABI's table, or EINVAL.
pub(crate) fn decode<T: Copy>(table: &[(i32, T)], code: i32) -> WasiResult<T> {
    table
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, v)| *v)
        .ok_or(WasiError::Inval)
}

pub(crate) fn encode<T: Copy + PartialEq>(table: &[(i32, T)], value: T) -> i32 {
    table.iter().find(|(_, v)| *v == value).unwrap().0
}

/// What a host's socket extension provides.
pub(crate) trait Backend {
    fn open(family: AddressFamily, ty: SocketType) -> WasiResult<i32>;
    fn bind(fd: i32, addr: &SocketAddr) -> WasiResult<()>;
    fn connect(fd: i32, addr: &SocketAddr) -> WasiResult<()>;
    fn listen(fd: i32, backlog: i32) -> WasiResult<()>;
    fn local_addr(fd: i32) -> WasiResult<SocketAddr>;
    fn peer_addr(fd: i32) -> WasiResult<SocketAddr>;
    /// Returns the byte count and the sender.
    fn recv_from(fd: i32, iovs: &[WasiIovec], flags: u32) -> WasiResult<(u32, SocketAddr)>;
    fn send_to(fd: i32, iovs: &[WasiIovec], flags: u32, to: &SocketAddr) -> WasiResult<u32>;
    /// Resolves `node` and `service`: at most `max` results, and how many
    /// the host found.
    fn resolve(
        node: &str,
        service: &str,
        family: AddressFamily,
        ty: SocketType,
        max: usize,
    ) -> WasiResult<(Vec<Resolved>, usize)>;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "socket-wamr")] {
        pub(crate) type Host = wamr::host::Wamr;
    } else if #[cfg(feature = "socket-wasmedge")] {
        pub(crate) type Host = wasmedge::host::WasmEdge;
    } else {
        pub(crate) type Host = NoHost;

        /// The backend of a build without a host feature.
        pub(crate) struct NoHost;

        impl Backend for NoHost {
            fn open(_: AddressFamily, _: SocketType) -> WasiResult<i32> {
                Err(WasiError::NotSup)
            }
            fn bind(_: i32, _: &SocketAddr) -> WasiResult<()> {
                Err(WasiError::NotSup)
            }
            fn connect(_: i32, _: &SocketAddr) -> WasiResult<()> {
                Err(WasiError::NotSup)
            }
            fn listen(_: i32, _: i32) -> WasiResult<()> {
                Err(WasiError::NotSup)
            }
            fn local_addr(_: i32) -> WasiResult<SocketAddr> {
                Err(WasiError::NotSup)
            }
            fn peer_addr(_: i32) -> WasiResult<SocketAddr> {
                Err(WasiError::NotSup)
            }
            fn recv_from(_: i32, _: &[WasiIovec], _: u32) -> WasiResult<(u32, SocketAddr)> {
                Err(WasiError::NotSup)
            }
            fn send_to(_: i32, _: &[WasiIovec], _: u32, _: &SocketAddr) -> WasiResult<u32> {
                Err(WasiError::NotSup)
            }
            fn resolve(
                _: &str,
                _: &str,
                _: AddressFamily,
                _: SocketType,
                _: usize,
            ) -> WasiResult<(Vec<Resolved>, usize)> {
                Err(WasiError::NotSup)
            }
        }
    }
}

/// Handles a socket extension call and returns the errno for the guest. A
/// WASI function reports failure through its errno, never by trapping.
pub(crate) fn call(
    ext: SocketExt,
    wasi: &PassthroughWasiImpl,
    memory: &MemAddr,
    params: &[Val],
) -> i32 {
    let result = match ext {
        SocketExt::OpenWamr => wamr::guest::open(memory, params),
        SocketExt::BindWamr => wamr::guest::bind(memory, params),
        SocketExt::ConnectWamr => wamr::guest::connect(memory, params),
        SocketExt::AddrLocal => wamr::guest::addr_local(memory, params),
        SocketExt::AddrRemote => wamr::guest::addr_remote(memory, params),
        SocketExt::RecvFromWamr => wamr::guest::recv_from(memory, params),
        SocketExt::SendToWamr => wamr::guest::send_to(memory, params),
        SocketExt::Listen => listen(params),
        SocketExt::Close => wamr::guest::close(wasi, params),
        SocketExt::AddrResolve => wamr::guest::addr_resolve(memory, params),
        SocketExt::GetAddrInfo => wasmedge::guest::getaddrinfo(memory, params),
        SocketExt::OpenWasmEdge => wasmedge::guest::open(memory, params),
        SocketExt::BindWasmEdge => wasmedge::guest::bind(memory, params),
        SocketExt::ConnectWasmEdge => wasmedge::guest::connect(memory, params),
        SocketExt::AcceptV1 => wasmedge::guest::accept_v1(wasi, memory, params),
        SocketExt::RecvFromV1 => wasmedge::guest::recv_from_v1(memory, params),
        SocketExt::RecvFromV2 => wasmedge::guest::recv_from_v2(memory, params),
        SocketExt::SendToWasmEdge => wasmedge::guest::send_to(memory, params),
        SocketExt::GetLocalAddrV1 => wasmedge::guest::local_addr_v1(memory, params),
        SocketExt::GetLocalAddrV2 => wasmedge::guest::local_addr_v2(memory, params),
        SocketExt::GetPeerAddrV1 => wasmedge::guest::peer_addr_v1(memory, params),
        SocketExt::GetPeerAddrV2 => wasmedge::guest::peer_addr_v2(memory, params),
        _ => Err(WasiError::NotSup),
    };
    result.map_or_else(|e| e.to_errno(), |()| 0)
}

/// `sock_listen(fd, backlog)`: the same signature on both hosts, and no
/// guest memory to read, so it needs no per-ABI frontend.
fn listen(params: &[Val]) -> WasiResult<()> {
    Host::listen(param_i32(params, 0)?, param_i32(params, 1)?)
}

pub(crate) fn param_i32(params: &[Val], i: usize) -> WasiResult<i32> {
    params
        .get(i)
        .and_then(|v| v.to_i32().ok())
        .ok_or(WasiError::Inval)
}

/// The iovec array that parameters 1 and 2 describe, with host pointers.
pub(crate) fn iovecs(memory: &MemAddr, params: &[Val]) -> WasiResult<Vec<WasiIovec>> {
    let mem = memory.get_memory_direct_access();
    collect_iovecs(
        mem,
        param_i32(params, 1)? as u32,
        param_i32(params, 2)? as u32,
    )
}

pub(crate) fn read_array<const N: usize>(memory: &MemAddr, ptr: i32) -> WasiResult<[u8; N]> {
    let mem = memory.get_memory_direct_access();
    guest_range(ptr as u32 as usize, N)
        .and_then(|r| mem.data.get(r))
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(WasiError::Fault)
}

pub(crate) fn read_bytes(memory: &MemAddr, ptr: i32, len: usize) -> WasiResult<Vec<u8>> {
    let mem = memory.get_memory_direct_access();
    guest_range(ptr as u32 as usize, len)
        .and_then(|r| mem.data.get(r))
        .map(<[u8]>::to_vec)
        .ok_or(WasiError::Fault)
}

pub(crate) fn write_bytes(memory: &MemAddr, ptr: i32, bytes: &[u8]) -> WasiResult<()> {
    guest_range(ptr as u32 as usize, bytes.len())
        .filter(|r| r.end <= memory.data_len())
        .ok_or(WasiError::Fault)?;
    memory.store_bytes(ptr, bytes);
    Ok(())
}

/// The NUL-terminated string at `ptr`.
pub(crate) fn read_cstr(memory: &MemAddr, ptr: i32) -> WasiResult<String> {
    let mem = memory.get_memory_direct_access();
    let start = ptr as u32 as usize;
    let rest = mem.data.get(start..).ok_or(WasiError::Fault)?;
    let len = rest.iter().position(|&b| b == 0).ok_or(WasiError::Fault)?;
    String::from_utf8(rest[..len].to_vec()).map_err(|_| WasiError::Inval)
}

/// The `len` bytes at `ptr` as text, less any trailing NUL.
pub(crate) fn read_str(memory: &MemAddr, ptr: i32, len: usize) -> WasiResult<String> {
    let mut bytes = read_bytes(memory, ptr, len)?;
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    String::from_utf8(bytes).map_err(|_| WasiError::Inval)
}

/// Reads a `#[repr(C)]` record with no implicit padding from guest memory.
pub(crate) fn read_struct<T: Copy>(memory: &MemAddr, ptr: i32) -> WasiResult<T> {
    let bytes = read_bytes(memory, ptr, std::mem::size_of::<T>())?;
    Ok(unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const T) })
}

/// Writes a `#[repr(C)]` record with no implicit padding to guest memory.
pub(crate) fn write_struct<T: Copy>(memory: &MemAddr, ptr: i32, value: &T) -> WasiResult<()> {
    let bytes = unsafe {
        std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
    };
    write_bytes(memory, ptr, bytes)
}
