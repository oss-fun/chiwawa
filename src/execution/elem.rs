use super::{func::FuncAddr, module::*, value::Ref};
use crate::structure::types::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ElemAddr(Rc<RefCell<ElemInst>>);
#[derive(Debug)]
pub struct ElemInst {
    pub _type_: RefType,
    pub _elem: Vec<Ref>,
}

impl ElemAddr {
    pub fn new(type_: &RefType, funcs: &Vec<FuncAddr>, init: &Vec<i32>) -> ElemAddr {
        let elem: Vec<Ref> = init
            .into_iter()
            .map(|i| Ref::FuncAddr(funcs.get_by_idx(FuncIdx(*i as u32)).clone()))
            .collect();
        ElemAddr(Rc::new(RefCell::new(ElemInst {
            _type_: type_.clone(),
            _elem: elem,
        })))
    }
}
