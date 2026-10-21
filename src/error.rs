use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuntimeError {
    #[error("Execution Failed: {0}")]
    ExecutionFailed(&'static str),
    #[error("Instantiate Failed")]
    InstantiateFailed,
    #[error("Export Function is not Found")]
    ExportFuncNotFound,
    #[error("Instruction Failed")]
    InstructionFailed,
    #[error("Invalid Conversion to Integer")]
    InvalidConversionToInt,
    #[error("Integer Overflow")]
    IntegerOverflow,
    #[error("Link Failed")]
    LinkError,
    #[error("Invalid Handler Index")]
    InvalidHandlerIndex,
    #[error("Memory Instance Not Found")]
    MemoryNotFound,
    #[error("Invalid Wasm Binary: {0}")]
    InvalidWasm(&'static str),
    #[error("Type Mismatch")]
    TypeMismatch,
    #[error("Invalid Parameter Count for Function Call")]
    InvalidParameterCount,
    #[error("Host Function Call is Unimplemented")]
    UnimplementedHostFunction,
    #[error("Table Instance Not Found")]
    TableNotFound,
    #[error("Indirect Call Type Mismatch")]
    IndirectCallTypeMismatch,
    #[error("Unreachable Executed")]
    Unreachable,
    #[error("Undefined Element")]
    UndefinedElement,
    #[error("Uninitialized Element")]
    UninitializedElement,
    #[error("Out of Bounds Table Access")]
    OutOfBoundsTableAccess,
    #[error("Out of Bounds Memory Access")]
    OutOfBoundsMemoryAccess,
    #[error("Invalid Constant Expression")]
    InvalidConstantExpression,
    #[error("Invalid Data Segment Index")]
    InvalidDataSegmentIndex,
    #[error("Unaligned Atomic Access")]
    UnalignedAtomicAccess,
    #[error("Atomic Wait on Unshared Memory")]
    AtomicWaitOnUnsharedMemory,

    // Migration Errors
    #[error("Serialization Error: {0}")]
    SerializationError(String),
    #[error("Deserialization Error: {0}")]
    DeserializationError(String),
    #[error("Checkpoint Save Error: {0}")]
    CheckpointSaveError(String),
    #[error("Checkpoint Load Error: {0}")]
    CheckpointLoadError(String),
    #[error("Checkpoint Requested")]
    CheckpointRequested,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParserError {
    #[error("Invalid Version")]
    VersionError,
    #[error("Unsupported OP Code in Global Section Init Expr at Offset: {offset}")]
    InitExprUnsupportedOPCodeError { offset: usize },
    #[error("Unexpected Else operator found")]
    UnexpectedElse,
    #[error("Unexpected End operator found")]
    UnexpectedEnd,
    #[error("Invalid Wasm: {0}")]
    InvalidWasm(&'static str),
    #[error("Not a core module")]
    NotAModule,
    #[error("Not a component")]
    NotAComponent,
    #[error("Invalid component: {0}")]
    InvalidComponent(String),
    #[error("Unsupported component feature: {0}")]
    UnsupportedComponentFeature(&'static str),
    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),
    #[error("Unsupported WASI function: {0}")]
    UnsupportedWasiFunction(String),
    #[error("WASI function type mismatch: {function} expected {expected:?} but got {actual:?}")]
    WasiFunctionTypeMismatch {
        function: String,
        expected: String,
        actual: String,
    },
}
