use super::value::Externval;

#[derive(Clone)]
pub struct ExportInst {
    pub name: String,
    pub value: Externval,
}