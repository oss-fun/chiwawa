use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::value::Ref;


pub struct TableAddr(Rc<RefCell<TableInst>>);
pub struct TableInst {
    pub type_: TableType,
    pub elem: Vec<Ref>,
}