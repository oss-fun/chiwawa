use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::{value::Ref, func::FuncAddr, module::*};

pub struct ElemAddr(Rc<RefCell<ElemInst>>);
pub struct ElemInst {
    pub type_: RefType,
    pub elem: Vec<Ref>,
    
}

impl ElemAddr{
    pub fn new(type_: &RefType, funcs: &Vec<FuncAddr>, init: i32) -> ElemAddr{
        let func_addr = funcs.get_by_idx(FuncIdx(init as u32)).clone();
        ElemAddr(Rc::new(RefCell::new(
            ElemInst{
                type_: type_.clone(),
                elem: vec![Ref::FuncAddr(func_addr)],
            }
        )))
    }
}