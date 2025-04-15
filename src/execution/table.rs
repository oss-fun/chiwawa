use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::{func::FuncAddr, module::*};

#[derive(Clone, Debug)]
pub struct TableAddr(Rc<RefCell<TableInst>>);
#[derive(Debug)]
pub struct TableInst {
    pub _type_: TableType,
    pub elem: Vec<Option<FuncAddr>>,
}

impl TableAddr{
    pub fn new(type_: &TableType) -> TableAddr{
        TableAddr(Rc::new(RefCell::new(
            TableInst{
                _type_:type_.clone(), 
                elem: {
                    let min = type_.0.min as usize;
                    let mut vec = Vec::with_capacity(min);
                    vec.resize(min, None);
                    vec
                }
            }
        )))
    }
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>){
        let addr_self = &mut self.0.borrow_mut();
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] = Some(funcs.get_by_idx(FuncIdx(*f as u32)).clone());
        }
    }
    pub fn get(&self, i: usize) -> Option<FuncAddr>{
        let inst = self.0.borrow();
        if i < inst.elem.len() as usize {
            inst.elem[i].clone()
        } else {
            None
        }
    }
}
