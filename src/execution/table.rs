use super::{func::FuncAddr, module::*, value::*};
use crate::error::RuntimeError;
use crate::structure::types::*;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};

#[derive(Clone, Debug)]
pub struct TableAddr(Arc<RwLock<TableInst>>);
#[derive(Debug)]
pub struct TableInst {
    pub _type_: TableType,
    pub elem: Vec<Val>,
}

impl TableAddr {
    pub fn new(type_: &TableType) -> TableAddr {
        TableAddr(Arc::new(RwLock::new(TableInst {
            _type_: type_.clone(),
            elem: {
                let min = type_.0.min as usize;
                let mut vec = Vec::with_capacity(min);
                vec.resize(min, Val::Ref(Ref::RefNull));
                vec
            },
        })))
    }
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>) {
        let mut addr_self = self.0.write().expect("RwLock poisoned");
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] =
                Val::Ref(Ref::FuncAddr(funcs.get_by_idx(FuncIdx(*f as u32)).clone()));
        }
    }
    pub fn get(&self, i: usize) -> Val {
        let inst = self.0.read().expect("RwLock poisoned");
        if i < inst.elem.len() as usize {
            inst.elem[i].clone()
        } else {
            Val::Ref(Ref::RefNull)
        }
    }

    pub fn set(&self, i: usize, val: Val) -> Result<(), RuntimeError> {
        let mut inst = self.0.write().expect("RwLock poisoned");
        if i < inst.elem.len() as usize {
            inst.elem[i] = val;
            Ok(())
        } else {
            Err(RuntimeError::InvalidTableIndex)
        }
    }

    pub fn get_func_addr(&self, i: usize) -> Option<FuncAddr> {
        let inst = self.0.read().expect("RwLock poisoned");
        if i < inst.elem.len() as usize {
            match &inst.elem[i] {
                Val::Ref(Ref::FuncAddr(func_addr)) => Some(func_addr.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn set_elements(&self, elems: Vec<Option<FuncAddr>>) -> Result<(), RuntimeError> {
        let mut guard = self
            .0
            .write()
            .map_err(|_| RuntimeError::ExecutionFailed("Table RwLock poisoned"))?;
        guard.elem = elems
            .into_iter()
            .map(|opt_func| match opt_func {
                Some(func_addr) => Val::Ref(Ref::FuncAddr(func_addr)),
                None => Val::Ref(Ref::RefNull),
            })
            .collect();
        Ok(())
    }

    pub fn read_lock(
        &self,
    ) -> Result<RwLockReadGuard<TableInst>, PoisonError<RwLockReadGuard<'_, TableInst>>> {
        self.0.read()
    }
}
