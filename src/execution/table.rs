use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::{value::Ref, func::FuncAddr, module::*};

#[derive(Clone)]
pub struct TableAddr(Rc<RefCell<TableInst>>);
pub struct TableInst {
    pub type_: TableType,
    pub elem: Vec<Ref>,
}

impl TableAddr{
    pub fn new(type_: &TableType) -> TableAddr{
        TableAddr(Rc::new(RefCell::new(
            TableInst{
                type_:type_.clone(), 
                elem: Vec::with_capacity(type_.0.min as usize)
            }
        )))
    }
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>){
        let addr_self = &mut self.0.borrow_mut();
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] = Ref::FuncAddr(funcs.get_by_idx(FuncIdx(*f as u32)).clone());
        }
    }
}