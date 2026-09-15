//! Socket extensions of WAMR and WasmEdge.
//!
//! A guest may use either extension. The host Chiwawa itself calls is chosen
//! at build time by the `socket-wamr` or `socket-wasmedge` feature.

#[cfg(all(feature = "socket-wamr", feature = "socket-wasmedge", not(docsrs)))]
compile_error!("socket-wamr and socket-wasmedge select different hosts; enable one");

use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::structure::module::SocketExt;
use crate::wasi::{WasiError, WasiResult};

pub fn call(_ext: SocketExt, _memory: &MemAddr, _params: &[Val]) -> WasiResult<i32> {
    // No backend yet: every build answers ENOTSUP.
    Ok(WasiError::NotSup.to_errno())
}
