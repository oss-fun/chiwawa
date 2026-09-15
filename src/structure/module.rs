//! WebAssembly module structure definitions.
//!
//! This module defines the structure of a parsed WebAssembly module, including
//! functions, tables, memories, globals, and imports/exports.
//!
//! ## Module Components
//!
//! A WebAssembly module consists of:
//! - **Types**: Function signatures shared across the module
//! - **Functions**: Code with local variables and a type reference
//! - **Tables**: Collections of function references for indirect calls
//! - **Memories**: Linear memory regions
//! - **Globals**: Global variables
//! - **Elements**: Table initialization data
//! - **Data**: Memory initialization data
//! - **Imports/Exports**: Module interface

use crate::execution::ir::{self, ProcessedInstr};
use crate::shared::Shared;
use crate::structure::instructions::*;
use crate::structure::types::*;
use std::cell::UnsafeCell;

/// Function definition within a module.
///
/// Contains the type signature, local variables, and preprocessed instruction body.
#[derive(Clone, Debug)]
pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<(u32, ValueType)>,
    pub body: Shared<Vec<ProcessedInstr>>,
    pub reg_allocation: Option<crate::execution::regs::RegAllocation>,
    pub handlers: Shared<HandlerTable>,
    /// Immediates too wide to encode inline (i64/f64). 32-bit immediates are
    /// carried in the instruction itself.
    pub wide_consts: Box<[u64]>,
}

/// The checkpoint monitor fills it to stop the interpreter
#[derive(Debug)]
pub struct HandlerTable(UnsafeCell<Vec<ir::Handler>>);

unsafe impl Send for HandlerTable {}
unsafe impl Sync for HandlerTable {}

impl HandlerTable {
    pub fn new(handlers: Vec<ir::Handler>) -> Self {
        HandlerTable(UnsafeCell::new(handlers))
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ir::Handler {
        unsafe { (*self.0.get()).as_ptr() }
    }

    pub fn fill(&self, handler: ir::Handler) {
        use std::sync::atomic::{AtomicPtr, Ordering};
        let entries = unsafe { &mut *self.0.get() };
        let base = entries.as_mut_ptr() as *mut AtomicPtr<()>;
        let target = handler as *mut ();
        for i in 0..entries.len() {
            unsafe { (*base.add(i)).store(target, Ordering::Relaxed) };
        }
    }
}

/// Table definition.
#[derive(Clone)]
pub struct Table {
    pub type_: TableType,
}

/// Memory definition.
#[derive(Clone)]
pub struct Mem {
    pub type_: MemType,
}

/// Global variable definition.
#[derive(Clone)]
pub struct Global {
    pub type_: GlobalType,
    pub init: Expr,
}

/// Element segment for table initialization.
pub struct Elem {
    pub type_: RefType,
    pub init: Option<Vec<Expr>>,
    pub idxes: Option<Vec<FuncIdx>>,
    pub mode: ElemMode,
    pub table_idx: Option<TableIdx>,
    pub offset: Option<Expr>,
}

/// Element segment mode.
#[derive(Debug, PartialEq)]
pub enum ElemMode {
    Passive,
    Active,
    Declarative,
}

/// Data segment for memory initialization.
pub struct Data {
    pub init: Vec<Byte>,
    pub mode: DataMode,
    pub memory: Option<MemIdx>,
    pub offset: Option<Expr>,
}

/// Data segment mode.
#[derive(Debug, PartialEq)]
pub enum DataMode {
    Passive,
    Active,
}

/// Start function specification.
pub struct Start {
    pub func: FuncIdx,
}

/// Import declaration.
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

/// Import descriptor specifying what is being imported.
#[derive(PartialEq, Debug)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
    WasiFunc(WasiFuncType),
}

/// WASI function types for passthrough implementation.
#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum WasiFuncType {
    ProcExit,
    FdWrite,
    FdRead,
    RandomGet,
    FdPrestatGet,
    FdPrestatDirName,
    FdClose,
    EnvironGet,
    EnvironSizesGet,
    ArgsGet,
    ArgsSizesGet,
    ClockTimeGet,
    ClockResGet,
    SchedYield,
    FdFdstatGet,
    PathOpen,
    FdSeek,
    FdTell,
    FdSync,
    FdFilestatGet,
    FdReaddir,
    FdPread,
    FdDatasync,
    FdFdstatSetFlags,
    FdFilestatSetSize,
    FdPwrite,
    PathCreateDirectory,
    PathFilestatGet,
    PathFilestatSetTimes,
    PathReadlink,
    PathRemoveDirectory,
    PathUnlinkFile,
    PollOneoff,
    ProcRaise,
    FdAdvise,
    FdAllocate,
    FdFdstatSetRights,
    FdRenumber,
    FdFilestatSetTimes,
    PathLink,
    PathRename,
    PathSymlink,
    SockAccept,
    SockRecv,
    SockSend,
    SockShutdown,
    /// `wasi::thread-spawn` from the wasi-threads proposal, imported from the
    /// `wasi` module rather than `wasi_snapshot_preview1`. Handled by
    /// `src/wasi/threads.rs`, not by passthrough.
    ThreadSpawn,
    /// A socket extension of a host runtime
    SocketExt(SocketExt),
}

/// Socket functions WAMR and WasmEdge add under `wasi_snapshot_preview1`.
/// Both hosts share some names with different shapes.
#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum SocketExt {
    Listen,
    OpenWamr,
    BindWamr,
    ConnectWamr,
    AddrLocal,
    AddrRemote,
    AddrResolve,
    RecvFromWamr,
    SendToWamr,
    Close,
    SetOptWamr(WamrSockOpt),
    GetOptWamr(WamrSockOpt),
    OpenWasmEdge,
    BindWasmEdge,
    ConnectWasmEdge,
    AcceptV1,
    RecvFromV1,
    RecvFromV2,
    SendToWasmEdge,
    GetLocalAddrV1,
    GetLocalAddrV2,
    GetPeerAddrV1,
    GetPeerAddrV2,
    GetAddrInfo,
    SetSockOpt,
    GetSockOpt,
}

impl SocketExt {
    /// WAMR from libc_wasi_wrapper.c, WasmEdge from wasifunc.h.
    pub fn func_type(&self) -> FuncType {
        match self {
            SocketExt::Listen => errno_func(vec![I32; 2]),
            SocketExt::OpenWamr => errno_func(vec![I32; 4]),
            SocketExt::BindWamr => errno_func(vec![I32; 2]),
            SocketExt::ConnectWamr => errno_func(vec![I32; 2]),
            SocketExt::AddrLocal => errno_func(vec![I32; 2]),
            SocketExt::AddrRemote => errno_func(vec![I32; 2]),
            SocketExt::AddrResolve => errno_func(vec![I32; 6]),
            SocketExt::RecvFromWamr => errno_func(vec![I32; 6]),
            SocketExt::SendToWamr => errno_func(vec![I32; 6]),
            SocketExt::Close => errno_func(vec![I32; 1]),
            SocketExt::SetOptWamr(opt) => errno_func(opt.set_params()),
            SocketExt::GetOptWamr(opt) => errno_func(opt.get_params()),
            SocketExt::OpenWasmEdge => errno_func(vec![I32; 3]),
            SocketExt::BindWasmEdge => errno_func(vec![I32; 3]),
            SocketExt::ConnectWasmEdge => errno_func(vec![I32; 3]),
            SocketExt::AcceptV1 => errno_func(vec![I32; 2]),
            SocketExt::RecvFromV1 => errno_func(vec![I32; 7]),
            SocketExt::RecvFromV2 => errno_func(vec![I32; 8]),
            SocketExt::SendToWasmEdge => errno_func(vec![I32; 7]),
            SocketExt::GetLocalAddrV1 => errno_func(vec![I32; 4]),
            SocketExt::GetLocalAddrV2 => errno_func(vec![I32; 3]),
            SocketExt::GetPeerAddrV1 => errno_func(vec![I32; 4]),
            SocketExt::GetPeerAddrV2 => errno_func(vec![I32; 3]),
            SocketExt::GetAddrInfo => errno_func(vec![I32; 8]),
            SocketExt::SetSockOpt => errno_func(vec![I32; 5]),
            SocketExt::GetSockOpt => errno_func(vec![I32; 5]),
        }
    }
}

/// A WAMR socket option. WAMR has one import per option; WasmEdge passes the
/// option as an argument, so it needs no counterpart.
#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum WamrSockOpt {
    ReuseAddr,
    ReusePort,
    KeepAlive,
    TcpNoDelay,
    TcpQuickAck,
    TcpFastopenConnect,
    Broadcast,
    IpTtl,
    IpMulticastTtl,
    Ipv6Only,
    RecvBufSize,
    SendBufSize,
    TcpKeepIdle,
    TcpKeepIntvl,
    RecvTimeout,
    SendTimeout,
    Linger,
    IpMulticastLoop,
    IpAddMembership,
    IpDropMembership,
}

impl WamrSockOpt {
    /// Each option with its setter and, where WAMR has one, getter name.
    pub const ALL: [(WamrSockOpt, &'static str, Option<&'static str>); 20] = [
        (
            WamrSockOpt::ReuseAddr,
            "sock_set_reuse_addr",
            Some("sock_get_reuse_addr"),
        ),
        (
            WamrSockOpt::ReusePort,
            "sock_set_reuse_port",
            Some("sock_get_reuse_port"),
        ),
        (
            WamrSockOpt::KeepAlive,
            "sock_set_keep_alive",
            Some("sock_get_keep_alive"),
        ),
        (
            WamrSockOpt::TcpNoDelay,
            "sock_set_tcp_no_delay",
            Some("sock_get_tcp_no_delay"),
        ),
        (
            WamrSockOpt::TcpQuickAck,
            "sock_set_tcp_quick_ack",
            Some("sock_get_tcp_quick_ack"),
        ),
        (
            WamrSockOpt::TcpFastopenConnect,
            "sock_set_tcp_fastopen_connect",
            Some("sock_get_tcp_fastopen_connect"),
        ),
        (
            WamrSockOpt::Broadcast,
            "sock_set_broadcast",
            Some("sock_get_broadcast"),
        ),
        (
            WamrSockOpt::IpTtl,
            "sock_set_ip_ttl",
            Some("sock_get_ip_ttl"),
        ),
        (
            WamrSockOpt::IpMulticastTtl,
            "sock_set_ip_multicast_ttl",
            Some("sock_get_ip_multicast_ttl"),
        ),
        (
            WamrSockOpt::Ipv6Only,
            "sock_set_ipv6_only",
            Some("sock_get_ipv6_only"),
        ),
        (
            WamrSockOpt::RecvBufSize,
            "sock_set_recv_buf_size",
            Some("sock_get_recv_buf_size"),
        ),
        (
            WamrSockOpt::SendBufSize,
            "sock_set_send_buf_size",
            Some("sock_get_send_buf_size"),
        ),
        (
            WamrSockOpt::TcpKeepIdle,
            "sock_set_tcp_keep_idle",
            Some("sock_get_tcp_keep_idle"),
        ),
        (
            WamrSockOpt::TcpKeepIntvl,
            "sock_set_tcp_keep_intvl",
            Some("sock_get_tcp_keep_intvl"),
        ),
        (
            WamrSockOpt::RecvTimeout,
            "sock_set_recv_timeout",
            Some("sock_get_recv_timeout"),
        ),
        (
            WamrSockOpt::SendTimeout,
            "sock_set_send_timeout",
            Some("sock_get_send_timeout"),
        ),
        (
            WamrSockOpt::Linger,
            "sock_set_linger",
            Some("sock_get_linger"),
        ),
        (
            WamrSockOpt::IpMulticastLoop,
            "sock_set_ip_multicast_loop",
            Some("sock_get_ip_multicast_loop"),
        ),
        (
            WamrSockOpt::IpAddMembership,
            "sock_set_ip_add_membership",
            None,
        ),
        (
            WamrSockOpt::IpDropMembership,
            "sock_set_ip_drop_membership",
            None,
        ),
    ];

    fn set_params(&self) -> Vec<ValueType> {
        match self {
            WamrSockOpt::RecvTimeout | WamrSockOpt::SendTimeout => vec![I32, I64],
            WamrSockOpt::Linger
            | WamrSockOpt::IpMulticastLoop
            | WamrSockOpt::IpAddMembership
            | WamrSockOpt::IpDropMembership => vec![I32, I32, I32],
            _ => vec![I32, I32],
        }
    }

    fn get_params(&self) -> Vec<ValueType> {
        match self {
            WamrSockOpt::Linger | WamrSockOpt::IpMulticastLoop => vec![I32, I32, I32],
            _ => vec![I32, I32],
        }
    }
}

const I32: ValueType = ValueType::NumType(NumType::I32);
const I64: ValueType = ValueType::NumType(NumType::I64);

/// A WASI signature taking `params` and returning an errno.
fn errno_func(params: Vec<ValueType>) -> FuncType {
    FuncType {
        params,
        results: vec![I32],
    }
}

impl WasiFuncType {
    /// WASI関数の期待される関数型を返す
    pub fn expected_func_type(&self) -> FuncType {
        match self {
            WasiFuncType::ProcExit => FuncType {
                params: vec![ValueType::NumType(NumType::I32)],
                results: vec![],
            },
            WasiFuncType::FdWrite => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdRead => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::RandomGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdPrestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdPrestatDirName => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdClose => FuncType {
                params: vec![ValueType::NumType(NumType::I32)],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::EnvironGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::EnvironSizesGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ArgsGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ArgsSizesGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ClockTimeGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I64),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ClockResGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::SchedYield => FuncType {
                params: vec![],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdFdstatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::PathOpen => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd (base directory)
                    ValueType::NumType(NumType::I32), // dirflags
                    ValueType::NumType(NumType::I32), // path ptr
                    ValueType::NumType(NumType::I32), // path len
                    ValueType::NumType(NumType::I32), // oflags
                    ValueType::NumType(NumType::I64), // fs_rights_base
                    ValueType::NumType(NumType::I64), // fs_rights_inheriting
                    ValueType::NumType(NumType::I32), // fdflags
                    ValueType::NumType(NumType::I32), // opened_fd ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdSeek => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // whence
                    ValueType::NumType(NumType::I32), // newoffset_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns errno
            },
            WasiFuncType::FdTell => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // offset_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdSync => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // filestat_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdReaddir => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // buf_ptr
                    ValueType::NumType(NumType::I32), // buf_len
                    ValueType::NumType(NumType::I64), // cookie
                    ValueType::NumType(NumType::I32), // buf_used_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdPread => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // iovs_ptr
                    ValueType::NumType(NumType::I32), // iovs_len
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // nread_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdDatasync => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFdstatSetFlags => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatSetSize => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // size
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdPwrite => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // iovs_ptr
                    ValueType::NumType(NumType::I32), // iovs_len
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // nwritten_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathCreateDirectory => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathFilestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I32), // filestat_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathFilestatSetTimes => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I64), // atim
                    ValueType::NumType(NumType::I64), // mtim
                    ValueType::NumType(NumType::I32), // fst_flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathReadlink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I32), // buf_ptr
                    ValueType::NumType(NumType::I32), // buf_len
                    ValueType::NumType(NumType::I32), // buf_used_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathRemoveDirectory => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathUnlinkFile => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PollOneoff => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // in_ptr
                    ValueType::NumType(NumType::I32), // out_ptr
                    ValueType::NumType(NumType::I32), // nsubscriptions
                    ValueType::NumType(NumType::I32), // nevents_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::ProcRaise => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // signal
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdAdvise => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I64), // len
                    ValueType::NumType(NumType::I32), // advice
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdAllocate => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I64), // len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFdstatSetRights => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // fs_rights_base
                    ValueType::NumType(NumType::I64), // fs_rights_inheriting
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdRenumber => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // to
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatSetTimes => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // atim
                    ValueType::NumType(NumType::I64), // mtim
                    ValueType::NumType(NumType::I32), // fst_flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathLink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_fd
                    ValueType::NumType(NumType::I32), // old_flags
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // new_fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathRename => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_fd
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // new_fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathSymlink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockAccept => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // fd_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockRecv => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // ri_data_ptr
                    ValueType::NumType(NumType::I32), // ri_data_len
                    ValueType::NumType(NumType::I32), // ri_flags
                    ValueType::NumType(NumType::I32), // ro_datalen_ptr
                    ValueType::NumType(NumType::I32), // ro_flags_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockSend => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // si_data_ptr
                    ValueType::NumType(NumType::I32), // si_data_len
                    ValueType::NumType(NumType::I32), // si_flags
                    ValueType::NumType(NumType::I32), // so_datalen_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockShutdown => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // how
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::ThreadSpawn => FuncType {
                params: vec![ValueType::NumType(NumType::I32)], // start_arg
                results: vec![ValueType::NumType(NumType::I32)], // Thread id, or negative errno
            },
            WasiFuncType::SocketExt(ext) => ext.func_type(),
        }
    }

    pub fn to_func_type(&self) -> FuncType {
        self.expected_func_type()
    }
}

/// Export declaration.
pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

/// Export descriptor specifying what is being exported.
#[derive(Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

/// A parsed WebAssembly module.
///
/// Contains all sections of a WebAssembly module after parsing and preprocessing.
pub struct Module {
    _name: String,
    /// Function type signatures.
    pub types: Shared<Vec<FuncType>>,
    /// Function definitions (including imported functions).
    pub funcs: Vec<Func>,
    /// Table definitions.
    pub tables: Vec<Table>,
    /// Memory definitions.
    pub mems: Vec<Mem>,
    /// Global variable definitions.
    pub globals: Vec<Global>,
    /// Element segments.
    pub elems: Vec<Elem>,
    /// Data segments.
    pub datas: Vec<Data>,
    /// Optional start function.
    pub start: Option<Start>,
    /// Import declarations.
    pub imports: Vec<Import>,
    /// Number of imported functions.
    pub num_imported_funcs: usize,
    /// Current code section index during parsing.
    pub code_index: usize,
    /// Export declarations.
    pub exports: Vec<Export>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            _name: name.to_string(),
            types: Shared::new(Vec::new()),
            funcs: Vec::new(),
            tables: Vec::new(),
            mems: Vec::new(),
            globals: Vec::new(),
            elems: Vec::new(),
            datas: Vec::new(),
            start: None,
            imports: Vec::new(),
            num_imported_funcs: 0,
            code_index: 0,
            exports: Vec::new(),
        }
    }
}
