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
    #[error("Divide by Zero")]
    ZeroDivideError,
    #[error("Invalid Conversion to Integer")]
    InvalidConversionToInt,
    #[error("Integer Overflow")]
    IntegerOverflow,
    #[error("Link Failed")]
    LinkError,
    #[error("Unreachable Code Reached")]
    Unreachable,
    #[error("Stack Error: {0}")]
    StackError(&'static str),
    #[error("Value Stack Underflow")]
    ValueStackUnderflow,
    #[error("Invalid Operand for Instruction")]
    InvalidOperand,
    #[error("Invalid Handler Index")]
    InvalidHandlerIndex,
    #[error("Unimplemented Instruction Reached")]
    UnimplementedInstruction,
    #[error("Unimplemented")]
    Unimplemented,
    #[error("Module Instance Reference Lost")]
    ModuleInstanceGone,
    #[error("Memory Instance Not Found")]
    MemoryNotFound,
    #[error("Memory Access Out Of Bounds")]
    MemoryOutOfBounds,
    #[error("Local Variable Index Out Of Bounds")]
    LocalIndexOutOfBounds,
    #[error("Global Variable Index Out Of Bounds")]
    GlobalIndexOutOfBounds,
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
    #[error("Type Not Found in Module")]
    TypeNotFound,
    #[error("Indirect Call Type Mismatch")]
    IndirectCallTypeMismatch,
    #[error("Uninitialized Element in Table")]
    UninitializedElement,
    #[error("Invalid Constant Expression")]
    InvalidConstantExpression,
    #[error("Trap")]
    Trap,

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
    #[error("Restore Error: {0}")]
    RestoreError(String),
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
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum StackError {
    #[error("Stack Underflow")]
    StackUnderflow,
    #[error("Value Type Mismatch")]
    ValueTypeMismatch,
    #[error("Invalid Label Stack Index: {0}")]
    InvalidLabelStackIndex(usize),
    #[error("Invalid Frame Stack Index: {0}")]
    InvalidFrameStackIndex(usize),
    #[error("Invalid Local Index: {0}")]
    InvalidLocalIndex(usize),
    #[error("Invalid Target Label Stack Index for Branch: {0}")]
    InvalidBranchTargetIndex(usize),
    #[error("Frame Stack Underflow")]
    FrameStackUnderflow,
    #[error("Label Stack Underflow")]
    LabelStackUnderflow,
    #[error("Empty Operand Stack during result transfer")]
    EmptyOperandStackForResult,
    #[error("Attempted to pop from empty label stack")]
    PopEmptyLabelStack,
}
