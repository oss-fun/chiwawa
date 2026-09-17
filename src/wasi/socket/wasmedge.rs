//! WasmEdge's socket ABI. Signatures come from wasifunc.h, layouts from
//! api.hpp and environ.h. V2 is the second revision of some functions; the
//! guest side accepts both, the host side calls V2.

pub(crate) mod guest;
#[cfg(feature = "socket-wasmedge")]
pub(crate) mod host;
pub(crate) mod layout;
