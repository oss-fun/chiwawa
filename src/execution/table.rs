//! Table instances for indirect function calls.

use super::{
    func::FuncAddr,
    module::*,
    value::{self, Val},
};
use crate::error::RuntimeError;
use crate::structure::types::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

/// Reference-counted handle to a table instance.
#[derive(Clone, Debug)]
pub struct TableAddr(Rc<RefCell<TableInst>>);

/// Table instance holding function references.
#[derive(Debug)]
pub struct TableInst {
    pub _type_: TableType,
    pub elem: Vec<Val>,
}

impl TableAddr {
    /// Creates a new table initialized with null references.
    pub fn new(type_: &TableType) -> TableAddr {
        TableAddr(Rc::new(RefCell::new(TableInst {
            _type_: *type_,
            elem: {
                let min = type_.0.min as usize;
                let mut vec = Vec::with_capacity(min);
                vec.resize(min, Val::Ref(value::Ref::RefNull));
                vec
            },
        })))
    }
    /// Initializes table elements from function indices.
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>) {
        let mut addr_self = self.0.borrow_mut();
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] = Val::Ref(value::Ref::FuncAddr(
                funcs.get_by_idx(FuncIdx(*f as u32)).clone(),
            ));
        }
    }
    /// Gets element at index. Out-of-bounds panics; host runtime traps.
    pub fn get(&self, i: usize) -> Val {
        self.0.borrow().elem[i].clone()
    }

    /// Sets element at index. Out-of-bounds panics; host runtime traps.
    pub fn set(&self, i: usize, val: Val) {
        self.0.borrow_mut().elem[i] = val;
    }

    /// Fills n elements starting at index with the given value.
    /// Out-of-bounds panics; host runtime traps.
    pub fn fill(&self, i: usize, val: Val, n: usize) {
        let mut inst = self.0.borrow_mut();
        for idx in i..i + n {
            inst.elem[idx] = val.clone();
        }
    }

    /// Gets function address at index for call_indirect.
    /// Out-of-bounds panics; returns None for non-FuncAddr references (e.g. RefNull),
    /// caller is expected to delegate the null-reference trap to the host via panic.
    pub fn get_func_addr(&self, i: usize) -> Option<FuncAddr> {
        let inst = self.0.borrow();
        match &inst.elem[i] {
            Val::Ref(value::Ref::FuncAddr(func_addr)) => Some(func_addr.clone()),
            _ => None,
        }
    }

    /// Replaces all elements (used during restore).
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

    /// Returns a borrow of the underlying table instance.
    pub fn read_lock(&self) -> Ref<TableInst> {
        self.0.borrow()
    }
}
