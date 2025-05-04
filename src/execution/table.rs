use super::{func::FuncAddr, module::*};
use crate::structure::types::*;
use crate::error::RuntimeError;
use std::sync::{Arc, RwLock, RwLockReadGuard, PoisonError};

#[derive(Clone, Debug)]
pub struct TableAddr(Arc<RwLock<TableInst>>);
#[derive(Debug)]
pub struct TableInst {
    pub _type_: TableType,
    pub elem: Vec<Option<FuncAddr>>,
}

impl TableAddr {
    pub fn new(type_: &TableType) -> TableAddr {
        TableAddr(Arc::new(RwLock::new(TableInst {
            _type_: type_.clone(),
            elem: {
                let min = type_.0.min as usize;
                let mut vec = Vec::with_capacity(min);
                vec.resize(min, None);
                vec
            },
        })))
    }
    pub fn init(&self, offset: usize, funcs: &Vec<FuncAddr>, init: &Vec<i32>) {
        let mut addr_self = self.0.write().expect("RwLock poisoned");
        for (index, f) in init.iter().enumerate() {
            addr_self.elem[index + offset] = Some(funcs.get_by_idx(FuncIdx(*f as u32)).clone());
        }
    }
    pub fn get(&self, i: usize) -> Option<FuncAddr> {
        let inst = self.0.read().expect("RwLock poisoned");
        if i < inst.elem.len() as usize {
            inst.elem[i].clone()
        } else {
            None
        }
    }


    pub fn set_elements(&self, elems: Vec<Option<FuncAddr>>) -> Result<(), RuntimeError> {
        let mut guard = self.0.write()
            .map_err(|_| RuntimeError::ExecutionFailed("Table RwLock poisoned"))?;
        guard.elem = elems;
        Ok(())
    }

    pub fn read_lock(
        &self,
    ) -> Result<RwLockReadGuard<TableInst>, PoisonError<RwLockReadGuard<'_, TableInst>>> {
        self.0.read()
    }
}
