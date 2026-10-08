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
/// Both hosts share some names with different signatures.
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

const I32: ValueType = ValueType::I32;
const I64: ValueType = ValueType::I64;

/// A WASI signature taking `params` and returning an errno.
fn errno_func(params: Vec<ValueType>) -> FuncType {
    FuncType {
        params,
        results: vec![I32],
    }
}

impl WasiFuncType {
    pub fn expected_func_type(&self) -> FuncType {
        match self {
            WasiFuncType::ProcExit => FuncType {
                params: vec![I32],
                results: vec![],
            },
            WasiFuncType::SchedYield => errno_func(vec![]),
            // Thread id, or a negative errno.
            WasiFuncType::ThreadSpawn
            | WasiFuncType::FdClose
            | WasiFuncType::FdSync
            | WasiFuncType::FdDatasync
            | WasiFuncType::ProcRaise => errno_func(vec![I32]),
            WasiFuncType::RandomGet
            | WasiFuncType::FdPrestatGet
            | WasiFuncType::EnvironGet
            | WasiFuncType::EnvironSizesGet
            | WasiFuncType::ArgsGet
            | WasiFuncType::ArgsSizesGet
            | WasiFuncType::ClockResGet
            | WasiFuncType::FdFdstatGet
            | WasiFuncType::FdTell
            | WasiFuncType::FdFilestatGet
            | WasiFuncType::FdFdstatSetFlags
            | WasiFuncType::FdRenumber
            | WasiFuncType::SockShutdown => errno_func(vec![I32; 2]),
            WasiFuncType::FdPrestatDirName
            | WasiFuncType::PathCreateDirectory
            | WasiFuncType::PathRemoveDirectory
            | WasiFuncType::PathUnlinkFile
            | WasiFuncType::SockAccept => errno_func(vec![I32; 3]),
            WasiFuncType::FdWrite | WasiFuncType::FdRead | WasiFuncType::PollOneoff => {
                errno_func(vec![I32; 4])
            }
            WasiFuncType::PathFilestatGet | WasiFuncType::SockSend | WasiFuncType::PathSymlink => {
                errno_func(vec![I32; 5])
            }
            WasiFuncType::PathReadlink | WasiFuncType::PathRename | WasiFuncType::SockRecv => {
                errno_func(vec![I32; 6])
            }
            WasiFuncType::PathLink => errno_func(vec![I32; 7]),
            WasiFuncType::ClockTimeGet => errno_func(vec![I32, I64, I32]),
            WasiFuncType::FdFilestatSetSize => errno_func(vec![I32, I64]),
            WasiFuncType::FdAllocate | WasiFuncType::FdFdstatSetRights => {
                errno_func(vec![I32, I64, I64])
            }
            WasiFuncType::FdAdvise | WasiFuncType::FdFilestatSetTimes => {
                errno_func(vec![I32, I64, I64, I32])
            }
            WasiFuncType::FdSeek => errno_func(vec![I32, I64, I32, I32]),
            WasiFuncType::FdReaddir | WasiFuncType::FdPread | WasiFuncType::FdPwrite => {
                errno_func(vec![I32, I32, I32, I64, I32])
            }
            WasiFuncType::PathFilestatSetTimes => {
                errno_func(vec![I32, I32, I32, I32, I64, I64, I32])
            }
            WasiFuncType::PathOpen => errno_func(vec![I32, I32, I32, I32, I32, I64, I64, I32, I32]),
            WasiFuncType::SocketExt(ext) => ext.func_type(),
        }
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
            exports: Vec::new(),
        }
    }
}
