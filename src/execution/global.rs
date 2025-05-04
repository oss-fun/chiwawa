use super::value::Val;
use crate::error::RuntimeError;
use crate::structure::types::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct GlobalAddr(Arc<RwLock<GlobalInst>>);
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalInst {
    pub _type_: GlobalType,
    pub value: Val,
}

impl GlobalAddr {
    pub fn new(type_: &GlobalType, value: Val) -> GlobalAddr {
        GlobalAddr(Arc::new(RwLock::new(GlobalInst {
            _type_: type_.clone(),
            value: value,
        })))
    }

    pub fn get(&self) -> Val {
        self.0.read().expect("RwLock poisoned").value.clone()
    }
    pub fn set(&self, value: Val) -> Result<(), RuntimeError> {
        let mut self_inst = self.0.write().expect("RwLock poisoned");
        if self_inst._type_.0 != Mut::Var {
            return Err(RuntimeError::InstructionFailed);
        }
        if self_inst.value.val_type() == value.val_type() {
            self_inst.value = value;
            return Ok(());
        }
        Err(RuntimeError::InstructionFailed)
    }

    pub fn set_value_unchecked(&self, value: Val) -> Result<(), RuntimeError> {
        let mut self_inst = self
            .0
            .write()
            .map_err(|_| RuntimeError::ExecutionFailed("Global RwLock poisoned"))?;
        if self_inst.value.val_type() == value.val_type() {
            self_inst.value = value;
            Ok(())
        } else {
            Err(RuntimeError::InstructionFailed)
        }
    }
}
