//! Record of the files and sockets the guest has open, and the fd numbers the guest knows them by.
//!
//! A restore is a new process, where the saved fd numbers name nothing.
//! The record holds what each open fd was opened with, so a restore can open it again.
//! The host picks the number of the new fd, so a table maps the number the guest remembers to it.

use crate::execution::mem::MemAddr;
use crate::execution::value::{Num, Val};
use crate::structure::module::{SocketExt, WasiFuncType};
use crate::wasi::passthrough::{
    __wasi_fd_close, __wasi_fd_fdstat_get, __wasi_fd_fdstat_set_flags, __wasi_fd_seek,
    __wasi_fd_tell, __wasi_path_open, __wasi_poll_oneoff, WasiIovec,
};
use crate::wasi::socket::{
    read_array, read_bytes, write_bytes, AddressFamily, Backend, Host, OptValue, Resolved, SockOpt,
    SocketType,
};
use crate::wasi::{Args, WasiError, WasiResult};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

const OFLAGS_EXCL: u16 = 1 << 2;
const OFLAGS_TRUNC: u16 = 1 << 3;
const WHENCE_SET: u32 = 0;

const SUBSCRIPTION_SIZE: usize = 48;
const EVENT_SIZE: usize = 32;
const SUBSCRIPTION_TAG: usize = 8;
const SUBSCRIPTION_FD: usize = 16;
const EVENTTYPE_FD_READ: u8 = 1;
const EVENTTYPE_FD_WRITE: u8 = 2;

/// The lowest number a guest fd alias may take, above stdio.
const FIRST_ALIAS: i32 = 3;

/// A file is what `path_open` returned an fd for.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct File {
    dirfd: u32,
    dirflags: u32,
    /// The path ends with a NUL byte.
    path: Vec<u8>,
    oflags: u16,
    rights_base: u64,
    rights_inheriting: u64,
    fdflags: u16,
    offset: u64,
}

/// A socket is what a socket extension's `sock_open` returned an fd for.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Socket {
    family: AddressFamily,
    ty: SocketType,
    opts: Vec<(SockOpt, OptValue)>,
    bound: Option<SocketAddr>,
    backlog: Option<i32>,
    peer: Option<SocketAddr>,
    fdflags: Option<u16>,
}

impl Socket {
    /// Records an option so that replaying `opts` in order gives the socket its current options.
    fn record_opt(&mut self, opt: SockOpt, value: OptValue) {
        let joins = |entry: &(SockOpt, OptValue)| *entry == (SockOpt::AddMembership, value);
        match opt {
            SockOpt::AddMembership if self.opts.iter().any(joins) => {}
            SockOpt::AddMembership => self.opts.push((opt, value)),
            SockOpt::DropMembership => self.opts.retain(|entry| !joins(entry)),
            _ => {
                self.opts.retain(|(set, _)| *set != opt);
                self.opts.push((opt, value));
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Kind {
    File(File),
    Socket(Socket),
}

/// A descriptor is one open fd and what a restore needs to open it again.
///
/// `fd` is the host's number in the record and the guest's number in a checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Descriptor {
    fd: i32,
    kind: Kind,
}

/// A mapping gives the host fd behind a guest fd number, or `None` for one that was not restored.
struct Mapping {
    guest: i32,
    host: Option<i32>,
}

struct State {
    /// The open descriptors, in the order they were opened.
    open: Vec<Descriptor>,
    /// A guest number missing from the table is the host's number.
    table: Vec<Mapping>,
}

static STATE: Mutex<State> = Mutex::new(State {
    open: Vec::new(),
    table: Vec::new(),
});

/// The table is empty until a restore, so calls skip the lock until one happened.
static TRANSLATING: AtomicBool = AtomicBool::new(false);

fn lock_state() -> MutexGuard<'static, State> {
    STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn check(errno: u16) -> WasiResult<()> {
    if errno == 0 {
        Ok(())
    } else {
        Err(WasiError::from_errno(errno))
    }
}

fn host_fd_is_open(fd: i32) -> bool {
    let mut stat = [0u8; 24];
    unsafe { __wasi_fd_fdstat_get(fd as u32, stat.as_mut_ptr()) == 0 }
}

impl State {
    fn opened(&mut self, fd: i32, kind: Kind) {
        self.open.retain(|descriptor| descriptor.fd != fd);
        self.open.push(Descriptor { fd, kind });
    }

    fn socket(&mut self, fd: i32) -> Option<&mut Socket> {
        match self.open.iter_mut().find(|descriptor| descriptor.fd == fd) {
            Some(Descriptor {
                kind: Kind::Socket(socket),
                ..
            }) => Some(socket),
            _ => None,
        }
    }

    /// The host fd behind a guest number, or `EBADF` for a number the guest was never given.
    fn to_host(&self, guest: i32) -> WasiResult<i32> {
        match self.table.iter().find(|mapping| mapping.guest == guest) {
            Some(mapping) => mapping.host.ok_or(WasiError::BadF),
            None if self.is_mapped_host(guest) => Err(WasiError::BadF),
            None => Ok(guest),
        }
    }

    fn to_guest(&self, host: i32) -> i32 {
        self.table
            .iter()
            .find(|mapping| mapping.host == Some(host))
            .map_or(host, |mapping| mapping.guest)
    }

    fn is_mapped_host(&self, host: i32) -> bool {
        self.table.iter().any(|mapping| mapping.host == Some(host))
    }

    fn is_guest_number(&self, number: i32) -> bool {
        self.table.iter().any(|mapping| mapping.guest == number)
    }

    /// The number the guest gets for a new host fd: the host's own, unless the guest already uses that number.
    fn adopt(&mut self, host: i32) -> i32 {
        if !self.is_guest_number(host) {
            return host;
        }
        let alias = (FIRST_ALIAS..)
            .find(|&n| !self.is_guest_number(n) && !self.is_mapped_host(n) && !host_fd_is_open(n))
            .unwrap_or(host);
        self.table.push(Mapping {
            guest: alias,
            host: Some(host),
        });
        alias
    }

    fn forget_host(&mut self, host: i32) {
        self.open.retain(|descriptor| descriptor.fd != host);
        self.table.retain(|mapping| mapping.host != Some(host));
    }
}

/// The positions of the fd parameters of a WASI call.
fn fd_params(func: &WasiFuncType) -> &'static [usize] {
    match func {
        WasiFuncType::FdWrite
        | WasiFuncType::FdRead
        | WasiFuncType::FdPrestatGet
        | WasiFuncType::FdPrestatDirName
        | WasiFuncType::FdClose
        | WasiFuncType::FdFdstatGet
        | WasiFuncType::PathOpen
        | WasiFuncType::FdSeek
        | WasiFuncType::FdTell
        | WasiFuncType::FdSync
        | WasiFuncType::FdFilestatGet
        | WasiFuncType::FdReaddir
        | WasiFuncType::FdPread
        | WasiFuncType::FdDatasync
        | WasiFuncType::FdFdstatSetFlags
        | WasiFuncType::FdFilestatSetSize
        | WasiFuncType::FdPwrite
        | WasiFuncType::PathCreateDirectory
        | WasiFuncType::PathFilestatGet
        | WasiFuncType::PathFilestatSetTimes
        | WasiFuncType::PathReadlink
        | WasiFuncType::PathRemoveDirectory
        | WasiFuncType::PathUnlinkFile
        | WasiFuncType::FdAdvise
        | WasiFuncType::FdAllocate
        | WasiFuncType::FdFdstatSetRights
        | WasiFuncType::FdFilestatSetTimes
        | WasiFuncType::SockAccept
        | WasiFuncType::SockRecv
        | WasiFuncType::SockSend
        | WasiFuncType::SockShutdown => &[0],
        WasiFuncType::FdRenumber => &[0, 1],
        WasiFuncType::PathLink | WasiFuncType::PathRename => &[0, 3],
        WasiFuncType::PathSymlink => &[2],
        WasiFuncType::ProcExit
        | WasiFuncType::RandomGet
        | WasiFuncType::EnvironGet
        | WasiFuncType::EnvironSizesGet
        | WasiFuncType::ArgsGet
        | WasiFuncType::ArgsSizesGet
        | WasiFuncType::ClockTimeGet
        | WasiFuncType::ClockResGet
        | WasiFuncType::SchedYield
        | WasiFuncType::PollOneoff
        | WasiFuncType::ProcRaise
        | WasiFuncType::ThreadSpawn => &[],
        WasiFuncType::SocketExt(ext) => match ext {
            SocketExt::OpenWamr
            | SocketExt::OpenWasmEdge
            | SocketExt::AddrResolve
            | SocketExt::GetAddrInfo => &[],
            SocketExt::Listen
            | SocketExt::BindWamr
            | SocketExt::ConnectWamr
            | SocketExt::AddrLocal
            | SocketExt::AddrRemote
            | SocketExt::RecvFromWamr
            | SocketExt::SendToWamr
            | SocketExt::Close
            | SocketExt::SetOptWamr(_)
            | SocketExt::GetOptWamr(_)
            | SocketExt::BindWasmEdge
            | SocketExt::ConnectWasmEdge
            | SocketExt::AcceptV1
            | SocketExt::RecvFromV1
            | SocketExt::RecvFromV2
            | SocketExt::SendToWasmEdge
            | SocketExt::GetLocalAddrV1
            | SocketExt::GetLocalAddrV2
            | SocketExt::GetPeerAddrV1
            | SocketExt::GetPeerAddrV2
            | SocketExt::SetSockOpt
            | SocketExt::GetSockOpt => &[0],
        },
    }
}

/// Runs a WASI call for the guest: `call` gets the host's fd numbers, and the record follows what the call did.
///
/// Every WASI call goes through this function, so no other code handles a guest fd number.
pub(crate) fn call_with_host_fds(
    func: &WasiFuncType,
    params: &[Val],
    memory: &MemAddr,
    call: impl FnOnce(&[Val]) -> WasiResult<i32>,
) -> WasiResult<i32> {
    let translated = host_params(func, params)?;
    let params = translated.as_deref().unwrap_or(params);
    let a = Args(params);
    if matches!(func, WasiFuncType::PollOneoff) && TRANSLATING.load(Ordering::Relaxed) {
        return poll_oneoff(memory, a.u32(0)?, a.u32(1)?, a.u32(2)?, a.u32(3)?);
    }
    let errno = call(params)?;
    if errno == 0 {
        record(func, &a, memory)?;
    }
    Ok(errno)
}

/// Updates the record and the table after a successful call, whose fd parameters are host numbers.
fn record(func: &WasiFuncType, a: &Args, memory: &MemAddr) -> WasiResult<()> {
    match func {
        WasiFuncType::PathOpen => file_opened(
            memory,
            a.u32(0)?,
            a.u32(1)?,
            a.u32(2)?,
            a.u32(3)?,
            a.u32(4)?,
            a.u64(5)?,
            a.u64(6)?,
            a.u32(7)?,
            a.u32(8)?,
        ),
        WasiFuncType::FdClose | WasiFuncType::SocketExt(SocketExt::Close) => {
            lock_state().forget_host(a.i32(0)?);
            Ok(())
        }
        WasiFuncType::FdRenumber => {
            renumbered(a.i32(0)?, a.i32(1)?);
            Ok(())
        }
        WasiFuncType::FdFdstatSetFlags => {
            flags_set(a.i32(0)?, a.u32(1)?);
            Ok(())
        }
        WasiFuncType::SockAccept => adopt_at(memory, a.u32(2)?),
        WasiFuncType::SocketExt(SocketExt::OpenWamr) => adopt_at(memory, a.u32(3)?),
        WasiFuncType::SocketExt(SocketExt::OpenWasmEdge) => adopt_at(memory, a.u32(2)?),
        WasiFuncType::SocketExt(SocketExt::AcceptV1) => adopt_at(memory, a.u32(1)?),
        _ => Ok(()),
    }
}

/// The parameters of a WASI call with every guest fd replaced by its host fd, or `None` when none differs.
fn host_params(func: &WasiFuncType, params: &[Val]) -> WasiResult<Option<Vec<Val>>> {
    if !TRANSLATING.load(Ordering::Relaxed) {
        return Ok(None);
    }
    let positions = fd_params(func);
    if positions.is_empty() {
        return Ok(None);
    }
    let state = lock_state();
    let mut translated = params.to_vec();
    for &position in positions {
        let guest = params
            .get(position)
            .and_then(|value| value.to_i32().ok())
            .ok_or(WasiError::Inval)?;
        translated[position] = Val::Num(Num::I32(state.to_host(guest)?));
    }
    Ok(Some(translated))
}

/// Gives the guest its number for the new host fd a call stored at `fd_ptr`.
fn adopt_at(memory: &MemAddr, fd_ptr: u32) -> WasiResult<()> {
    if !TRANSLATING.load(Ordering::Relaxed) {
        return Ok(());
    }
    let host = i32::from_le_bytes(read_array(memory, fd_ptr as i32)?);
    let guest = lock_state().adopt(host);
    if guest != host {
        write_bytes(memory, fd_ptr as i32, &guest.to_le_bytes())?;
    }
    Ok(())
}

/// `poll_oneoff` on a copy of the subscriptions, so the guest's memory keeps the guest's fd numbers.
fn poll_oneoff(
    memory: &MemAddr,
    in_ptr: u32,
    out_ptr: u32,
    nsubscriptions: u32,
    nevents_ptr: u32,
) -> WasiResult<i32> {
    let count = nsubscriptions as usize;
    let mut subscriptions = read_bytes(memory, in_ptr as i32, count * SUBSCRIPTION_SIZE)?;
    {
        let state = lock_state();
        for subscription in subscriptions.chunks_exact_mut(SUBSCRIPTION_SIZE) {
            let tag = subscription[SUBSCRIPTION_TAG];
            if tag != EVENTTYPE_FD_READ && tag != EVENTTYPE_FD_WRITE {
                continue;
            }
            let fd = &mut subscription[SUBSCRIPTION_FD..SUBSCRIPTION_FD + 4];
            let guest = i32::from_le_bytes(fd.try_into().unwrap());
            fd.copy_from_slice(&state.to_host(guest).unwrap_or(-1).to_le_bytes());
        }
    }
    let mut events = vec![0u8; count * EVENT_SIZE];
    let mut nevents = 0u32;
    let errno = unsafe {
        __wasi_poll_oneoff(
            subscriptions.as_ptr(),
            events.as_mut_ptr(),
            nsubscriptions,
            &mut nevents,
        )
    };
    if errno == 0 {
        let stored = (nevents as usize).min(count) * EVENT_SIZE;
        write_bytes(memory, out_ptr as i32, &events[..stored])?;
        write_bytes(memory, nevents_ptr as i32, &nevents.to_le_bytes())?;
    }
    Ok(errno as i32)
}

/// Records a successful `path_open`, whose parameters these are, with `dirfd` as the host's number.
fn file_opened(
    memory: &MemAddr,
    dirfd: u32,
    dirflags: u32,
    path_ptr: u32,
    path_len: u32,
    oflags: u32,
    rights_base: u64,
    rights_inheriting: u64,
    fdflags: u32,
    opened_fd_ptr: u32,
) -> WasiResult<()> {
    let mut path = read_bytes(memory, path_ptr as i32, path_len as usize)?;
    path.push(0);
    let fd = i32::from_le_bytes(read_array(memory, opened_fd_ptr as i32)?);
    lock_state().opened(
        fd,
        Kind::File(File {
            dirfd,
            dirflags,
            path,
            oflags: oflags as u16,
            rights_base,
            rights_inheriting,
            fdflags: fdflags as u16,
            offset: 0,
        }),
    );
    adopt_at(memory, opened_fd_ptr)
}

/// `Host`, with every call that shapes a socket recorded.
pub(crate) struct Recorded;

impl Backend for Recorded {
    fn open(family: AddressFamily, ty: SocketType) -> WasiResult<i32> {
        let fd = Host::open(family, ty)?;
        lock_state().opened(
            fd,
            Kind::Socket(Socket {
                family,
                ty,
                opts: Vec::new(),
                bound: None,
                backlog: None,
                peer: None,
                fdflags: None,
            }),
        );
        Ok(fd)
    }
    fn bind(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        Host::bind(fd, addr)?;
        let bound = match addr.port() {
            0 => Host::local_addr(fd).unwrap_or(*addr),
            _ => *addr,
        };
        if let Some(socket) = lock_state().socket(fd) {
            socket.bound = Some(bound);
        }
        Ok(())
    }
    fn connect(fd: i32, addr: &SocketAddr) -> WasiResult<()> {
        Host::connect(fd, addr)?;
        if let Some(socket) = lock_state().socket(fd) {
            socket.peer = Some(*addr);
        }
        Ok(())
    }
    fn listen(fd: i32, backlog: i32) -> WasiResult<()> {
        Host::listen(fd, backlog)?;
        if let Some(socket) = lock_state().socket(fd) {
            socket.backlog = Some(backlog);
        }
        Ok(())
    }
    fn local_addr(fd: i32) -> WasiResult<SocketAddr> {
        Host::local_addr(fd)
    }
    fn peer_addr(fd: i32) -> WasiResult<SocketAddr> {
        Host::peer_addr(fd)
    }
    fn recv_from(fd: i32, iovs: &[WasiIovec], flags: u32) -> WasiResult<(u32, SocketAddr)> {
        Host::recv_from(fd, iovs, flags)
    }
    fn send_to(fd: i32, iovs: &[WasiIovec], flags: u32, to: &SocketAddr) -> WasiResult<u32> {
        Host::send_to(fd, iovs, flags, to)
    }
    fn resolve(
        node: &str,
        service: &str,
        family: AddressFamily,
        ty: SocketType,
        max: usize,
    ) -> WasiResult<(Vec<Resolved>, usize)> {
        Host::resolve(node, service, family, ty, max)
    }
    fn set_opt(fd: i32, opt: SockOpt, value: OptValue) -> WasiResult<()> {
        Host::set_opt(fd, opt, value)?;
        if let Some(socket) = lock_state().socket(fd) {
            socket.record_opt(opt, value);
        }
        Ok(())
    }
    fn get_opt(fd: i32, opt: SockOpt) -> WasiResult<OptValue> {
        Host::get_opt(fd, opt)
    }
}

/// Records a successful `fd_fdstat_set_flags` on a host fd.
fn flags_set(fd: i32, flags: u32) {
    let mut state = lock_state();
    match state.open.iter_mut().find(|descriptor| descriptor.fd == fd) {
        Some(Descriptor {
            kind: Kind::File(file),
            ..
        }) => file.fdflags = flags as u16,
        Some(Descriptor {
            kind: Kind::Socket(socket),
            ..
        }) => socket.fdflags = Some(flags as u16),
        None => {}
    }
}

/// Records a successful `fd_renumber` on host fds, which closes `to` and moves `from` there.
fn renumbered(from: i32, to: i32) {
    let mut state = lock_state();
    state.open.retain(|descriptor| descriptor.fd != to);
    state.table.retain(|mapping| mapping.host != Some(from));
    if let Some(descriptor) = state.open.iter_mut().find(|d| d.fd == from) {
        descriptor.fd = to;
    }
}

/// The open descriptors as a checkpoint saves them: guest fd numbers, and each file's current offset.
pub fn snapshot() -> Vec<Descriptor> {
    let state = lock_state();
    state
        .open
        .iter()
        .map(|descriptor| {
            let mut saved = descriptor.clone();
            saved.fd = state.to_guest(descriptor.fd);
            if let Kind::File(file) = &mut saved.kind {
                let mut offset = 0;
                if unsafe { __wasi_fd_tell(descriptor.fd as u32, &mut offset) } == 0 {
                    file.offset = offset;
                }
                file.dirfd = state.to_guest(file.dirfd as i32) as u32;
            }
            saved
        })
        .collect()
}

/// Opens every saved descriptor again and maps its saved fd number to the new fd.
///
/// A descriptor that cannot be opened is reported, and the guest sees `EBADF` for its number.
pub fn restore(saved: &[Descriptor]) {
    if saved.is_empty() {
        return;
    }
    let mut state = lock_state();
    let mut restored = 0;
    for descriptor in saved {
        let reopened = match &descriptor.kind {
            Kind::File(file) => reopen_file(&state, file),
            Kind::Socket(socket) => reopen_socket(socket).map(|fd| (fd, descriptor.kind.clone())),
        };
        let host = match reopened {
            Ok((fd, kind)) => {
                state.opened(fd, kind);
                restored += 1;
                Some(fd)
            }
            Err(e) => {
                eprintln!(
                    "Warning: fd {} was not restored ({}): {:?}",
                    descriptor.fd, e, descriptor.kind
                );
                None
            }
        };
        if host != Some(descriptor.fd) {
            state.table.push(Mapping {
                guest: descriptor.fd,
                host,
            });
        }
    }
    TRANSLATING.store(!state.table.is_empty(), Ordering::Relaxed);
    println!("Descriptors restored: {} of {}.", restored, saved.len());
}

/// Opens `file` again without `O_EXCL` and `O_TRUNC`, which would fail on or empty the file it already wrote.
fn reopen_file(state: &State, file: &File) -> WasiResult<(i32, Kind)> {
    let dirfd = state.to_host(file.dirfd as i32)? as u32;
    let mut opened = 0;
    check(unsafe {
        __wasi_path_open(
            dirfd,
            file.dirflags,
            file.path.as_ptr(),
            file.oflags & !(OFLAGS_EXCL | OFLAGS_TRUNC),
            file.rights_base,
            file.rights_inheriting,
            file.fdflags,
            &mut opened,
        )
    })?;
    if file.offset != 0 {
        let mut offset = 0;
        let errno = unsafe { __wasi_fd_seek(opened, file.offset as i64, WHENCE_SET, &mut offset) };
        if errno != 0 {
            unsafe { __wasi_fd_close(opened) };
            return Err(WasiError::from_errno(errno));
        }
    }
    let file = File {
        dirfd,
        ..file.clone()
    };
    Ok((opened as i32, Kind::File(file)))
}

fn reopen_socket(socket: &Socket) -> WasiResult<i32> {
    let fd = Host::open(socket.family, socket.ty)?;
    let shaped = shape_socket(fd, socket);
    if shaped.is_err() {
        unsafe { __wasi_fd_close(fd as u32) };
    }
    shaped.map(|()| fd)
}

/// Applies the recorded options and addresses to a newly opened socket.
fn shape_socket(fd: i32, socket: &Socket) -> WasiResult<()> {
    for (opt, value) in &socket.opts {
        Host::set_opt(fd, *opt, *value)?;
    }
    if let Some(flags) = socket.fdflags {
        check(unsafe { __wasi_fd_fdstat_set_flags(fd as u32, flags as u32) })?;
    }
    if let Some(addr) = &socket.bound {
        Host::bind(fd, addr)?;
    }
    if let Some(backlog) = socket.backlog {
        Host::listen(fd, backlog)?;
    }
    if let Some(peer) = &socket.peer {
        Host::connect(fd, peer)?;
    }
    Ok(())
}
