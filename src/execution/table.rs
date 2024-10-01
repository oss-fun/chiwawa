use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::value::Ref;

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
}