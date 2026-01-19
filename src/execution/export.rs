//! Export instances mapping names to external values.

use super::value::Externval;

/// Named export binding.
#[derive(Clone)]
pub struct ExportInst {
    pub name: String,
    pub value: Externval,
}
