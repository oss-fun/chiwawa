use super::{
    func::FuncAddr,
    module::*,
    value::{self, Val},
};
use crate::error::RuntimeError;
use crate::structure::types::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct TableAddr(Rc<RefCell<TableInst>>);
#[derive(Debug)]
pub struct TableInst {
    pub _type_: TableType,
    pub elem: Vec<Val>,
}

impl TableAddr {
    pub fn new(type_: &TableType) -> TableAddr {
        TableAddr(Rc::new(RefCell::new(TableInst {
            _type_: type_.clone(),
            elem: {
                let min = type_.0.min as usize;
                let mut vec = Vec::with_capacity(min);
                vec.resize(min, Val::Ref(value::Ref::RefNull));
                vec
            },
        })))
    }
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>) {
        let mut addr_self = self.0.borrow_mut();
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] = Val::Ref(value::Ref::FuncAddr(
                funcs.get_by_idx(FuncIdx(*f as u32)).clone(),
            ));
        }
    }
    pub fn get(&self, i: usize) -> Val {
        let inst = self.0.borrow();
        if i < inst.elem.len() as usize {
            inst.elem[i].clone()
        } else {
            Val::Ref(value::Ref::RefNull)
        }
    }

    pub fn set(&self, i: usize, val: Val) -> Result<(), RuntimeError> {
        let mut inst = self.0.borrow_mut();
        if i < inst.elem.len() as usize {
            inst.elem[i] = val;
            Ok(())
        } else {
            Err(RuntimeError::InvalidTableIndex)
        }
    }

    pub fn get_func_addr(&self, i: usize) -> Option<FuncAddr> {
        let inst = self.0.borrow();
        if i < inst.elem.len() as usize {
            match &inst.elem[i] {
                Val::Ref(value::Ref::FuncAddr(func_addr)) => Some(func_addr.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_elements(&self, elems: Vec<Option<FuncAddr>>) -> Result<(), RuntimeError> {
        let mut guard = self.0.borrow_mut();
        guard.elem = elems
            .into_iter()
            .map(|opt_func| match opt_func {
                Some(func_addr) => Val::Ref(value::Ref::FuncAddr(func_addr)),
                None => Val::Ref(value::Ref::RefNull),
            })
            .collect();
        Ok(())
    }

    pub fn read_lock(&self) -> Ref<TableInst> {
        self.0.borrow()
    }
}
