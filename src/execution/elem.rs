use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::{value::Ref, func::FuncAddr, module::*};

pub struct ElemAddr(Rc<RefCell<ElemInst>>);
pub struct ElemInst {
    pub type_: RefType,
    pub elem: Vec<Ref>,
    
}

impl ElemAddr{
    pub fn new(type_: &RefType, funcs: &Vec<FuncAddr>, init: &Vec<i32>) -> ElemAddr{
        let elem :Vec<Ref> = init.into_iter().map(|i|Ref::FuncAddr(funcs.get_by_idx(FuncIdx(*i as u32)).clone())).collect();
        ElemAddr(Rc::new(RefCell::new(
            ElemInst{
                type_: type_.clone(),
                elem: elem,
            }
        )))
    }
}