use std::{rc::Rc, cell::*, rc::Weak};
use crate::structure::{types::*,module::*, instructions::Expr};
use super::value::Val; // Import Val directly
use super::module::*; // Keep module import
use crate::error::RuntimeError; // Import RuntimeError
#[cfg(feature = "fast")] // Conditionally import stackopt
use crate::execution::stackopt;
#[cfg(feature = "interp")] // Conditionally import stack
use crate::execution::stack;

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
    pub fn call(&self, params: Vec<Val>) -> Result<Vec<Val>, RuntimeError> {
        #[cfg(feature = "fast")]
        {
            let mut dtc_stacks = stackopt::Stacks::new(self, params)?;
            dtc_stacks.exec_instr()
        }
        #[cfg(all(not(feature = "fast"), feature = "interp"))]
        {
            let mut stack_stacks = stack::Stacks::new(self, params);
            loop {
                match stack_stacks.exec_instr() {
                    Ok(()) => { // Assuming Ok(()) means continue
                        // Check completion condition based on stack state
                        if stack_stacks.activation_frame_stack.len() == 1
                        && stack_stacks.activation_frame_stack.first().map_or(true, |f| f.label_stack.len() == 1)
                        && stack_stacks.activation_frame_stack.first().and_then(|f| f.label_stack.first()).map_or(true, |l| l.instrs.is_empty())
                        {
                            break;
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            // Return final value stack
            if let Some(mut last_frame) = stack_stacks.activation_frame_stack.pop() {
                if let Some(last_label) = last_frame.label_stack.pop() {
                            Ok(last_label.value_stack)
                        } else {
                            Err(RuntimeError::StackError("Final label stack empty"))
                        }
                    } else {
                         Err(RuntimeError::StackError("Final frame stack empty"))
                    }
                }
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
