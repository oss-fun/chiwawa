use std::sync::{Arc, RwLock};
use crate::structure::types::*;
use super::{value::Ref, func::FuncAddr, module::*};

#[derive(Clone, Debug)]
pub struct ElemAddr(Arc<RwLock<ElemInst>>);
#[derive(Debug)]
pub struct ElemInst {
    pub _type_: RefType,
    pub _elem: Vec<Ref>,
    
}

impl ElemAddr{
    pub fn new(type_: &RefType, funcs: &Vec<FuncAddr>, init: &Vec<i32>) -> ElemAddr{
        let elem :Vec<Ref> = init.into_iter().map(|i|Ref::FuncAddr(funcs.get_by_idx(FuncIdx(*i as u32)).clone())).collect();
        ElemAddr(Arc::new(RwLock::new(
            ElemInst{
                _type_: type_.clone(),
                _elem: elem,
            }
        )))
    }
}