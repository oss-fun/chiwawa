//! WAMR's socket ABI. Signatures come from libc_wasi_wrapper.c, layouts from
//! wasi_socket_ext.h.

pub(crate) mod guest;
#[cfg(feature = "socket-wamr")]
pub(crate) mod host;
pub(crate) mod layout;
