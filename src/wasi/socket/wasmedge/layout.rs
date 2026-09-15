//! The numbers and layouts WasmEdge defines, shared by the guest and host
//! sides. Pointers are `u32`: guest and host are both wasm32.

use crate::wasi::socket::{AddressFamily, SocketType};
use crate::wasi::{WasiError, WasiResult};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub(crate) const FAMILY_CODES: [(i32, AddressFamily); 3] = [
    (0, AddressFamily::Unspec),
    (1, AddressFamily::Inet4),
    (2, AddressFamily::Inet6),
];
pub(crate) const TYPE_CODES: [(i32, SocketType); 3] = [
    (0, SocketType::Any),
    (1, SocketType::Datagram),
    (2, SocketType::Stream),
];

/// `__wasi_address_t`: a pointer and a length. The buffer holds raw octets
/// (4 or 16 bytes), or in V2 this storage form: u16 family, then octets.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct WasiAddress {
    pub buf: u32,
    pub buf_len: u32,
}
pub(crate) const STORAGE_SIZE: usize = 128;
pub(crate) const FAMILY_INET4: u16 = 1;
pub(crate) const FAMILY_INET6: u16 = 2;

/// Reads the IP from an address buffer of any accepted length.
pub(crate) fn decode_ip(buf: &[u8]) -> WasiResult<IpAddr> {
    let (family, octets) = match buf.len() {
        4 => (FAMILY_INET4, buf),
        16 => (FAMILY_INET6, buf),
        STORAGE_SIZE => (u16::from_le_bytes([buf[0], buf[1]]), &buf[2..]),
        _ => return Err(WasiError::Inval),
    };
    match family {
        FAMILY_INET4 => Ok(Ipv4Addr::from(<[u8; 4]>::try_from(&octets[..4]).unwrap()).into()),
        FAMILY_INET6 => Ok(Ipv6Addr::from(<[u8; 16]>::try_from(&octets[..16]).unwrap()).into()),
        _ => Err(WasiError::AfNoSupport),
    }
}

pub(crate) fn decode_addr(buf: &[u8], port: u16) -> WasiResult<SocketAddr> {
    Ok(SocketAddr::new(decode_ip(buf)?, port))
}

/// Writes the IP into an address buffer in the form its length asks for.
pub(crate) fn encode_ip(buf: &mut [u8], ip: &IpAddr) -> WasiResult<()> {
    let (family, octets): (u16, Vec<u8>) = match ip {
        IpAddr::V4(v4) => (FAMILY_INET4, v4.octets().to_vec()),
        IpAddr::V6(v6) => (FAMILY_INET6, v6.octets().to_vec()),
    };
    let octets_at = match buf.len() {
        STORAGE_SIZE => {
            buf[0..2].copy_from_slice(&family.to_le_bytes());
            &mut buf[2..]
        }
        len if len >= octets.len() => buf,
        _ => return Err(WasiError::Inval),
    };
    octets_at[..octets.len()].copy_from_slice(&octets);
    Ok(())
}

/// V1's address getters report the family as 4 or 6.
pub(crate) fn v1_family_code(ip: &IpAddr) -> u32 {
    match ip {
        IpAddr::V4(_) => 4,
        IpAddr::V6(_) => 6,
    }
}

/// `__wasi_protocol_t`
const PROTOCOL_IP: u8 = 0;
const PROTOCOL_TCP: u8 = 1;
const PROTOCOL_UDP: u8 = 2;

/// The protocol a socket type implies.
pub(crate) fn protocol_code(ty: SocketType) -> u8 {
    match ty {
        SocketType::Any => PROTOCOL_IP,
        SocketType::Stream => PROTOCOL_TCP,
        SocketType::Datagram => PROTOCOL_UDP,
    }
}

/// `__wasi_addrinfo_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct AddrInfo {
    pub flags: u16,
    pub family: u8,
    pub socktype: u8,
    pub protocol: u8,
    pub _pad: [u8; 3],
    pub addrlen: u32,
    pub addr: u32,
    pub canonname: u32,
    pub canonname_len: u32,
    pub next: u32,
}

/// `__wasi_sockaddr_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SockAddr {
    pub family: u8,
    pub _pad: [u8; 3],
    pub data_len: u32,
    pub data: u32,
}

/// `sa_data` is a Linux `sockaddr_in` or `sockaddr_in6` after its family:
/// the port in network order, then the IPv4 address and 8 zero bytes, or 4
/// flow-info bytes, the IPv6 address and 4 scope bytes.
pub(crate) const SA_DATA_MAX: usize = 26;

/// The family byte, `sa_data` and its length for an address. `ai_addrlen`
/// is the length plus the 2 family bytes.
pub(crate) fn sa_data(addr: &SocketAddr) -> (u8, [u8; SA_DATA_MAX], usize) {
    let mut data = [0u8; SA_DATA_MAX];
    data[0..2].copy_from_slice(&addr.port().to_be_bytes());
    match addr.ip() {
        IpAddr::V4(v4) => {
            data[2..6].copy_from_slice(&v4.octets());
            (FAMILY_INET4 as u8, data, 14)
        }
        IpAddr::V6(v6) => {
            data[6..22].copy_from_slice(&v6.octets());
            (FAMILY_INET6 as u8, data, SA_DATA_MAX)
        }
    }
}

#[cfg(feature = "socket-wasmedge")]
pub(crate) fn decode_sa_data(family: u8, data: &[u8; SA_DATA_MAX]) -> WasiResult<SocketAddr> {
    let port = u16::from_be_bytes([data[0], data[1]]);
    let ip: IpAddr = match family as u16 {
        FAMILY_INET4 => Ipv4Addr::from(<[u8; 4]>::try_from(&data[2..6]).unwrap()).into(),
        FAMILY_INET6 => Ipv6Addr::from(<[u8; 16]>::try_from(&data[6..22]).unwrap()).into(),
        _ => return Err(WasiError::Inval),
    };
    Ok(SocketAddr::new(ip, port))
}
