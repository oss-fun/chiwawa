use std::{rc::Rc, cell::*, rc::Weak};
use crate::structure::{types::*,module::*, instructions::Expr};
use super::value::Val; // Import Val directly
use super::module::*; // Keep module import
use crate::error::RuntimeError; // Import RuntimeError

use std::fmt::{self, Debug}; // Import Debug and fmt

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


// Implementation for FuncAddr to hold the methods
impl FuncAddr {
    // The call method seems to depend on the 'interp' feature and stack module,
    // which might be causing the unresolved import error.
    // Commenting out the body for now.
    pub fn call(&self, _params: Vec<Val>) -> Result<Vec<Val>, RuntimeError> {
        // let mut stack = Stacks::new(&self, params);
        // loop{
        //     stack.exec_instr()?;
        //     /*Reached Dummy Stack Frame*/
        //     if stack.activation_frame_stack.len() == 1
        //     && stack.activation_frame_stack.first().unwrap().label_stack.len() == 1
        //     && stack.activation_frame_stack.first().unwrap().label_stack.first().unwrap().processed_instrs.is_empty(){
        //         break;
        //     }
        // }
        // Ok(stack.activation_frame_stack.pop().unwrap().label_stack.pop().unwrap().value_stack)
        Err(RuntimeError::Unimplemented) // Return unimplemented error for now
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
                        body: Expr(Vec::new())
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

// Removed the separate FuncInst impl block as `new` is now handled within `replace`
// or could be a separate associated function if needed elsewhere.
