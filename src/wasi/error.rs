//! WASI error codes and handling.
//!
//! This module defines WASI Preview 1 error codes as the [`WasiError`] enum,
//! with conversions to and from errno values.

use thiserror::Error;

/// Result type for WASI operations.
pub type WasiResult<T> = Result<T, WasiError>;

/// WASI Preview 1 error codes.
///
/// Each variant corresponds to a WASI errno value. The `to_errno()` and
/// `from_errno()` methods convert between the enum and numeric representations.
#[derive(Error, Debug, Clone)]
pub enum WasiError {
    #[error("Success")]
    Success, // ESUCCESS = 0

    #[error("Argument list too long")]
    E2Big, // E2BIG = 1

    #[error("Permission denied")]
    Acces, // EACCES = 2

    #[error("Address in use")]
    AddrInUse, // EADDRINUSE = 3

    #[error("Address not available")]
    AddrNotAvail, // EADDRNOTAVAIL = 4

    #[error("Address family not supported")]
    AfNoSupport, // EAFNOSUPPORT = 5

    #[error("Resource unavailable, try again")]
    Again, // EAGAIN = 6

    #[error("Connection already in progress")]
    Already, // EALREADY = 7

    #[error("Bad file descriptor")]
    BadF, // EBADF = 8

    #[error("Bad message")]
    BadMsg, // EBADMSG = 9

    #[error("Device or resource busy")]
    Busy, // EBUSY = 10

    #[error("Operation canceled")]
    Canceled, // ECANCELED = 11

    #[error("No child processes")]
    Child, // ECHILD = 12

    #[error("Connection aborted")]
    ConnAborted, // ECONNABORTED = 13

    #[error("Connection refused")]
    ConnRefused, // ECONNREFUSED = 14

    #[error("Connection reset")]
    ConnReset, // ECONNRESET = 15

    #[error("Resource deadlock would occur")]
    DeadLk, // EDEADLK = 16

    #[error("Destination address required")]
    DestAddrReq, // EDESTADDRREQ = 17

    #[error("Mathematics argument out of domain of function")]
    Dom, // EDOM = 18

    #[error("Reserved")]
    DQuot, // EDQUOT = 19

    #[error("File exists")]
    Exist, // EEXIST = 20

    #[error("Bad address")]
    Fault, // EFAULT = 21

    #[error("File too large")]
    FBig, // EFBIG = 22

    #[error("Host is unreachable")]
    HostUnreach, // EHOSTUNREACH = 23

    #[error("Identifier removed")]
    IdRm, // EIDRM = 24

    #[error("Illegal byte sequence")]
    IlSeq, // EILSEQ = 25

    #[error("Operation in progress")]
    InProgress, // EINPROGRESS = 26

    #[error("Interrupted function")]
    Intr, // EINTR = 27

    #[error("Invalid argument")]
    Inval, // EINVAL = 28

    #[error("I/O error")]
    Io, // EIO = 29

    #[error("Socket is connected")]
    IsConn, // EISCONN = 30

    #[error("Is a directory")]
    IsDir, // EISDIR = 31

    #[error("Too many levels of symbolic links")]
    Loop, // ELOOP = 32

    #[error("File descriptor value too large")]
    MFile, // EMFILE = 33

    #[error("Too many links")]
    MLink, // EMLINK = 34

    #[error("Message too large")]
    MsgSize, // EMSGSIZE = 35

    #[error("Reserved")]
    MultiHop, // EMULTIHOP = 36

    #[error("Filename too long")]
    NameTooLong, // ENAMETOOLONG = 37

    #[error("Network is down")]
    NetDown, // ENETDOWN = 38

    #[error("Connection aborted by network")]
    NetReset, // ENETRESET = 39

    #[error("Network unreachable")]
    NetUnreach, // ENETUNREACH = 40

    #[error("Too many files open in system")]
    NFile, // ENFILE = 41

    #[error("No buffer space available")]
    NoBufs, // ENOBUFS = 42

    #[error("No such device")]
    NoDev, // ENODEV = 43

    #[error("No such file or directory")]
    NoEnt, // ENOENT = 44

    #[error("Executable file format error")]
    NoExec, // ENOEXEC = 45

    #[error("No locks available")]
    NoLck, // ENOLCK = 46

    #[error("Reserved")]
    NoLink, // ENOLINK = 47

    #[error("Not enough space")]
    NoMem, // ENOMEM = 48

    #[error("No message of the desired type")]
    NoMsg, // ENOMSG = 49

    #[error("Protocol not available")]
    NoProtoOpt, // ENOPROTOOPT = 50

    #[error("No space left on device")]
    NoSpc, // ENOSPC = 51

    #[error("Function not supported")]
    NoSys, // ENOSYS = 52

    #[error("The socket is not connected")]
    NotConn, // ENOTCONN = 53

    #[error("Not a directory or a symbolic link to a directory")]
    NotDir, // ENOTDIR = 54

    #[error("Directory not empty")]
    NotEmpty, // ENOTEMPTY = 55

    #[error("State not recoverable")]
    NotRecoverable, // ENOTRECOVERABLE = 56

    #[error("Not a socket")]
    NotSock, // ENOTSOCK = 57

    #[error("Not supported, or operation not supported on socket")]
    NotSup, // ENOTSUP = 58

    #[error("Inappropriate I/O control operation")]
    NotTty, // ENOTTY = 59

    #[error("No such device or address")]
    NxIo, // ENXIO = 60

    #[error("Value too large to be stored in data type")]
    Overflow, // EOVERFLOW = 61

    #[error("Previous owner died")]
    OwnerDead, // EOWNERDEAD = 62

    #[error("Operation not permitted")]
    Perm, // EPERM = 63

    #[error("Broken pipe")]
    Pipe, // EPIPE = 64

    #[error("Protocol error")]
    Proto, // EPROTO = 65

    #[error("Protocol not supported")]
    ProtoNoSupport, // EPROTONOSUPPORT = 66

    #[error("Protocol wrong type for socket")]
    Prototype, // EPROTOTYPE = 67

    #[error("Result too large")]
    Range, // ERANGE = 68

    #[error("Read-only file system")]
    RoFs, // EROFS = 69

    #[error("Invalid seek")]
    SPipe, // ESPIPE = 70

    #[error("No such process")]
    Srch, // ESRCH = 71

    #[error("Reserved")]
    Stale, // ESTALE = 72

    #[error("Connection timed out")]
    TimedOut, // ETIMEDOUT = 73

    #[error("Text file busy")]
    TxtBsy, // ETXTBSY = 74

    #[error("Cross-device link")]
    XDev, // EXDEV = 75

    #[error("Extension: Capabilities insufficient")]
    NotCapable, // ENOTCAPABLE = 76

    #[error("Process exit requested with code {0}")]
    ProcessExit(i32),
}

impl WasiError {
    /// Convert errno value to WASI error (Chromium WASI errno.js)
    pub fn from_errno(errno: u16) -> Self {
        match errno {
            0 => WasiError::Success,
            1 => WasiError::E2Big,
            2 => WasiError::Acces,
            3 => WasiError::AddrInUse,
            4 => WasiError::AddrNotAvail,
            5 => WasiError::AfNoSupport,
            6 => WasiError::Again,
            7 => WasiError::Already,
            8 => WasiError::BadF,
            9 => WasiError::BadMsg,
            10 => WasiError::Busy,
            11 => WasiError::Canceled,
            12 => WasiError::Child,
            13 => WasiError::ConnAborted,
            14 => WasiError::ConnRefused,
            15 => WasiError::ConnReset,
            16 => WasiError::DeadLk,
            17 => WasiError::DestAddrReq,
            18 => WasiError::Dom,
            19 => WasiError::DQuot,
            20 => WasiError::Exist,
            21 => WasiError::Fault,
            22 => WasiError::FBig,
            23 => WasiError::HostUnreach,
            24 => WasiError::IdRm,
            25 => WasiError::IlSeq,
            26 => WasiError::InProgress,
            27 => WasiError::Intr,
            28 => WasiError::Inval,
            29 => WasiError::Io,
            30 => WasiError::IsConn,
            31 => WasiError::IsDir,
            32 => WasiError::Loop,
            33 => WasiError::MFile,
            34 => WasiError::MLink,
            35 => WasiError::MsgSize,
            36 => WasiError::MultiHop,
            37 => WasiError::NameTooLong,
            38 => WasiError::NetDown,
            39 => WasiError::NetReset,
            40 => WasiError::NetUnreach,
            41 => WasiError::NFile,
            42 => WasiError::NoBufs,
            43 => WasiError::NoDev,
            44 => WasiError::NoEnt,
            45 => WasiError::NoExec,
            46 => WasiError::NoLck,
            47 => WasiError::NoLink,
            48 => WasiError::NoMem,
            49 => WasiError::NoMsg,
            50 => WasiError::NoProtoOpt,
            51 => WasiError::NoSpc,
            52 => WasiError::NoSys,
            53 => WasiError::NotConn,
            54 => WasiError::NotDir,
            55 => WasiError::NotEmpty,
            56 => WasiError::NotRecoverable,
            57 => WasiError::NotSock,
            58 => WasiError::NotSup,
            59 => WasiError::NotTty,
            60 => WasiError::NxIo,
            61 => WasiError::Overflow,
            62 => WasiError::OwnerDead,
            63 => WasiError::Perm,
            64 => WasiError::Pipe,
            65 => WasiError::Proto,
            66 => WasiError::ProtoNoSupport,
            67 => WasiError::Prototype,
            68 => WasiError::Range,
            69 => WasiError::RoFs,
            70 => WasiError::SPipe,
            71 => WasiError::Srch,
            72 => WasiError::Stale,
            73 => WasiError::TimedOut,
            74 => WasiError::TxtBsy,
            75 => WasiError::XDev,
            76 => WasiError::NotCapable,
            _ => WasiError::Io, // Default to I/O error for unknown errno
        }
    }

    /// Convert WASI error to errno value (Chromium WASI errno.js)
    pub fn to_errno(&self) -> i32 {
        match self {
            WasiError::Success => 0,
            WasiError::E2Big => 1,
            WasiError::Acces => 2,
            WasiError::AddrInUse => 3,
            WasiError::AddrNotAvail => 4,
            WasiError::AfNoSupport => 5,
            WasiError::Again => 6,
            WasiError::Already => 7,
            WasiError::BadF => 8,
            WasiError::BadMsg => 9,
            WasiError::Busy => 10,
            WasiError::Canceled => 11,
            WasiError::Child => 12,
            WasiError::ConnAborted => 13,
            WasiError::ConnRefused => 14,
            WasiError::ConnReset => 15,
            WasiError::DeadLk => 16,
            WasiError::DestAddrReq => 17,
            WasiError::Dom => 18,
            WasiError::DQuot => 19,
            WasiError::Exist => 20,
            WasiError::Fault => 21,
            WasiError::FBig => 22,
            WasiError::HostUnreach => 23,
            WasiError::IdRm => 24,
            WasiError::IlSeq => 25,
            WasiError::InProgress => 26,
            WasiError::Intr => 27,
            WasiError::Inval => 28,
            WasiError::Io => 29,
            WasiError::IsConn => 30,
            WasiError::IsDir => 31,
            WasiError::Loop => 32,
            WasiError::MFile => 33,
            WasiError::MLink => 34,
            WasiError::MsgSize => 35,
            WasiError::MultiHop => 36,
            WasiError::NameTooLong => 37,
            WasiError::NetDown => 38,
            WasiError::NetReset => 39,
            WasiError::NetUnreach => 40,
            WasiError::NFile => 41,
            WasiError::NoBufs => 42,
            WasiError::NoDev => 43,
            WasiError::NoEnt => 44,
            WasiError::NoExec => 45,
            WasiError::NoLck => 46,
            WasiError::NoLink => 47,
            WasiError::NoMem => 48,
            WasiError::NoMsg => 49,
            WasiError::NoProtoOpt => 50,
            WasiError::NoSpc => 51,
            WasiError::NoSys => 52,
            WasiError::NotConn => 53,
            WasiError::NotDir => 54,
            WasiError::NotEmpty => 55,
            WasiError::NotRecoverable => 56,
            WasiError::NotSock => 57,
            WasiError::NotSup => 58,
            WasiError::NotTty => 59,
            WasiError::NxIo => 60,
            WasiError::Overflow => 61,
            WasiError::OwnerDead => 62,
            WasiError::Perm => 63,
            WasiError::Pipe => 64,
            WasiError::Proto => 65,
            WasiError::ProtoNoSupport => 66,
            WasiError::Prototype => 67,
            WasiError::Range => 68,
            WasiError::RoFs => 69,
            WasiError::SPipe => 70,
            WasiError::Srch => 71,
            WasiError::Stale => 72,
            WasiError::TimedOut => 73,
            WasiError::TxtBsy => 74,
            WasiError::XDev => 75,
            WasiError::NotCapable => 76,
            WasiError::ProcessExit(_) => 0,
        }
    }
}
