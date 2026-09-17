//! The numbers and layouts WAMR defines, shared by the guest and host sides.

use crate::structure::module::WamrSockOpt;
use crate::wasi::socket::{decode, encode, AddressFamily, Resolved, SockOpt, SocketType};
use crate::wasi::{WasiError, WasiResult};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

/// `__wasi_addr_t`: kind at 0; then IPv4 octets at 4 and port at 8, or IPv6
/// 16-bit groups at 4 and port at 20. Integers are little endian.
pub(crate) const ADDR_SIZE: usize = 24;
const KIND_IPV4: u32 = 0;
const KIND_IPV6: u32 = 1;

pub(crate) const FAMILY_CODES: [(i32, AddressFamily); 3] = [
    (0, AddressFamily::Inet4),
    (1, AddressFamily::Inet6),
    (2, AddressFamily::Unspec),
];
pub(crate) const TYPE_CODES: [(i32, SocketType); 3] = [
    (-1, SocketType::Any),
    (0, SocketType::Datagram),
    (1, SocketType::Stream),
];

pub(crate) fn decode_addr(bytes: &[u8; ADDR_SIZE]) -> WasiResult<SocketAddr> {
    let u16_at = |at: usize| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
    match u32::from_le_bytes(bytes[0..4].try_into().unwrap()) {
        KIND_IPV4 => {
            let octets: [u8; 4] = bytes[4..8].try_into().unwrap();
            Ok(SocketAddr::from((Ipv4Addr::from(octets), u16_at(8))))
        }
        KIND_IPV6 => {
            let mut groups = [0u16; 8];
            for (i, group) in groups.iter_mut().enumerate() {
                *group = u16_at(4 + i * 2);
            }
            Ok(SocketAddr::from((Ipv6Addr::from(groups), u16_at(20))))
        }
        _ => Err(WasiError::Inval),
    }
}

pub(crate) fn encode_addr(addr: &SocketAddr) -> [u8; ADDR_SIZE] {
    let mut bytes = [0u8; ADDR_SIZE];
    match addr {
        SocketAddr::V4(v4) => {
            bytes[0..4].copy_from_slice(&KIND_IPV4.to_le_bytes());
            bytes[4..8].copy_from_slice(&v4.ip().octets());
            bytes[8..10].copy_from_slice(&v4.port().to_le_bytes());
        }
        SocketAddr::V6(v6) => {
            bytes[0..4].copy_from_slice(&KIND_IPV6.to_le_bytes());
            for (i, group) in v6.ip().segments().iter().enumerate() {
                bytes[4 + i * 2..6 + i * 2].copy_from_slice(&group.to_le_bytes());
            }
            bytes[20..22].copy_from_slice(&v6.port().to_le_bytes());
        }
    }
    bytes
}

/// `__wasi_addr_info_hints_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Hints {
    pub ty: i32,
    pub family: i32,
    pub enabled: u8,
    pub _pad: [u8; 3],
}

impl Hints {
    #[cfg(feature = "socket-wamr")]
    pub(crate) fn new(family: AddressFamily, ty: SocketType) -> Self {
        Self {
            ty: encode(&TYPE_CODES, ty),
            family: encode(&FAMILY_CODES, family),
            enabled: (family != AddressFamily::Unspec || ty != SocketType::Any) as u8,
            _pad: [0; 3],
        }
    }

    pub(crate) fn decode(&self) -> WasiResult<(AddressFamily, SocketType)> {
        if self.enabled == 0 {
            return Ok((AddressFamily::Unspec, SocketType::Any));
        }
        Ok((
            decode(&FAMILY_CODES, self.family)?,
            decode(&TYPE_CODES, self.ty)?,
        ))
    }
}

/// `__wasi_addr_info_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct AddrInfo {
    pub addr: [u8; ADDR_SIZE],
    pub ty: i32,
}

impl AddrInfo {
    pub(crate) fn new(found: &Resolved) -> Self {
        Self {
            addr: encode_addr(&found.addr),
            ty: encode(&TYPE_CODES, found.ty),
        }
    }

    #[cfg(feature = "socket-wamr")]
    pub(crate) fn decode(&self) -> WasiResult<Resolved> {
        Ok(Resolved {
            addr: decode_addr(&self.addr)?,
            ty: decode(&TYPE_CODES, self.ty)?,
        })
    }
}

/// The common option a WAMR option function stands for
pub(crate) fn common_opt(opt: WamrSockOpt) -> WasiResult<SockOpt> {
    match opt {
        WamrSockOpt::ReuseAddr => Ok(SockOpt::ReuseAddr),
        WamrSockOpt::KeepAlive => Ok(SockOpt::KeepAlive),
        WamrSockOpt::Broadcast => Ok(SockOpt::Broadcast),
        WamrSockOpt::RecvBufSize => Ok(SockOpt::RecvBufSize),
        WamrSockOpt::SendBufSize => Ok(SockOpt::SendBufSize),
        WamrSockOpt::RecvTimeout => Ok(SockOpt::RecvTimeout),
        WamrSockOpt::SendTimeout => Ok(SockOpt::SendTimeout),
        WamrSockOpt::Linger => Ok(SockOpt::Linger),
        _ => Err(WasiError::NoProtoOpt),
    }
}
