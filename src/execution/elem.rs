use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::value::Ref;

pub struct ElemAddr(Rc<RefCell<ElemInst>>);
pub struct ElemInst {
    pub type_: RefType,
    pub elem: Vec<Ref>,
    
}