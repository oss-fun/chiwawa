//! Global variable instances.

use super::value::Val;
use crate::error::RuntimeError;
use crate::structure::types::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// Reference-counted handle to a global instance.
#[derive(Clone, Debug)]
pub struct GlobalAddr {
    global_inst: Rc<RefCell<GlobalInst>>,
}

/// Global variable instance with type and mutability info.
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalInst {
    pub _type_: GlobalType,
    pub value: Val,
}

impl GlobalAddr {
    /// Creates a new global with initial value.
    pub fn new(type_: &GlobalType, value: Val) -> GlobalAddr {
        GlobalAddr {
            global_inst: Rc::new(RefCell::new(GlobalInst {
                _type_: type_.clone(),
                value: value,
            })),
        }
    }

    /// Gets the current value.
    pub fn get(&self) -> Val {
        self.global_inst.borrow().value.clone()
    }

    /// Sets the value (type must match).
    pub fn set(&self, value: Val) -> Result<(), RuntimeError> {
        let mut self_inst = self.global_inst.borrow_mut();
        if self_inst.value.val_type() == value.val_type() {
            self_inst.value = value;
            Ok(())
        } else {
            Err(RuntimeError::InstructionFailed)
        }
    }
}
