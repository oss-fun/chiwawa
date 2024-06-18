use crate::types::*;
use crate::instructions::*;

pub struct Func {
    type_: TypeIdx,
    locals: Vec<ValueType>,
    body: Expr,
} 

pub struct Module {
    name: String,
    types: Vec<FuncType>,
    funcs: Vec<Func>,
}