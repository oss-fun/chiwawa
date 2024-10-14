use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Execution Failed")]
    ExecutionFailed,
    #[error("Instantiate Failed")]
    InstantiateFailed,
    #[error("Export Function is not Found")]
    ExportFuncNotFound,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Invalid Version")]
    VersionError,
    #[error("Unsupported OP Code in Global Section Init Expr at Offset: {offset}")]
    InitExprUnsupportedOPCodeError{offset: usize},
}