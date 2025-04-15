use std::{rc::Rc, cell::*, rc::Weak};
use crate::structure::{types::*,module::*};
use super::value::Val;
use super::module::*;
use crate::error::RuntimeError;
use crate::execution::stack::Stacks;
use std::fmt::{self, Debug};

#[derive(Clone, Debug)]
pub struct FuncAddr(Rc<RefCell<FuncInst>>);
pub enum FuncInst {
    RuntimeFunc{
        type_: FuncType,
        module: Weak<ModuleInst>,
        code: Func,
    },
    HostFunc{
        type_: FuncType,
        host_code: Rc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError>>,
    },
}

impl Debug for FuncInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncInst::RuntimeFunc { type_, module, code } => f
                .debug_struct("RuntimeFunc")
                .field("type_", type_)
                .field("module", module) // module is Weak<ModuleInst>, might need custom formatting if ModuleInst isn't Debug
                .field("code", code)
                .finish(),
            FuncInst::HostFunc { type_, host_code: _ } => f // Ignore host_code field
                .debug_struct("HostFunc")
                .field("type_", type_)
                .field("host_code", &"<host_code>") // Use placeholder
                .finish(),
        }
    }
}


impl FuncAddr {
    pub fn call(&self, params: Vec<Val>) -> Result<Vec<Val>, RuntimeError> {
        let mut dtc_stacks = Stacks::new(self, params)?;
        dtc_stacks.exec_instr()
    }

    pub fn borrow(&self) -> Ref<FuncInst> {
        self.0.borrow()
    }

    pub fn alloc_empty() -> FuncAddr{
        FuncAddr(
            Rc::new(RefCell::new(
                FuncInst::RuntimeFunc{
                    type_: FuncType{
                        params: Vec::new(),
                        results: Vec::new()
                    },
                    module: Weak::new(),
                    code: Func{
                        type_: TypeIdx(0),
                        locals: Vec::new(),
                        body: Vec::new(),
                        fixups: Vec::new(),
                        processed_cache: std::cell::RefCell::new(None), // Initialize cache here too
                    }
                }
            ))
        )
    }

    pub fn replace(&self, func: Func, module: Weak<ModuleInst>){
        let new_inst = FuncInst::RuntimeFunc{
            type_: module.upgrade().expect("Module weak ref expired").types.get_by_idx(func.type_.clone()).clone(),
            module: module,
            code: func,
        };
        *self.0.borrow_mut() = new_inst;
    }

    pub fn func_type(&self) ->FuncType{
        match &*self.0.borrow() {
            FuncInst::RuntimeFunc { type_, .. } => type_.clone(),
            FuncInst::HostFunc { type_, .. } => type_.clone(),
        }
    }
}
