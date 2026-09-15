//! Socket extensions of the host runtimes.
//!
//! WAMR and WasmEdge each add socket functions under `wasi_snapshot_preview1`
//! with their own names and shapes. The parser tells a guest's imports apart
//! per function ([`SocketExt`]). Which host Chiwawa itself runs on comes from
//! `--socket-host`: calling an import the host did not link traps, so the
//! host cannot be probed.

use crate::execution::mem::MemAddr;
use crate::execution::value::Val;
use crate::structure::module::SocketExt;
use crate::wasi::{WasiError, WasiResult};
use std::sync::OnceLock;

/// The host runtime whose socket extension Chiwawa may call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFlavor {
    Wamr,
    WasmEdge,
}

static HOST: OnceLock<HostFlavor> = OnceLock::new();

pub fn set_host(flavor: HostFlavor) {
    let _ = HOST.set(flavor);
}

pub fn host() -> Option<HostFlavor> {
    HOST.get().copied()
}

pub fn call(_ext: SocketExt, _memory: &MemAddr, _params: &[Val]) -> WasiResult<i32> {
    // No backend yet: every host answers ENOTSUP.
    Ok(WasiError::NotSup.to_errno())
}
