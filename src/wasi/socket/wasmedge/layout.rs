//! The numbers and layouts WasmEdge defines, shared by the guest and host
//! sides.

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

/// `__wasi_address_t` is a pointer and a length. The buffer holds raw octets
/// (4 or 16 bytes), or in V2 this storage form: u16 family, then octets.
pub(crate) const STORAGE_SIZE: usize = 128;
const FAMILY_INET4: u16 = 1;
const FAMILY_INET6: u16 = 2;

pub(crate) fn decode<T: Copy>(table: &[(i32, T)], code: i32) -> WasiResult<T> {
    table
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, v)| *v)
        .ok_or(WasiError::Inval)
}

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
