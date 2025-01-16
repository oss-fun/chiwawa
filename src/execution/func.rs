use std::{rc::Rc, cell::*, rc::Weak};
use crate::structure::{types::*,module::*, instructions::Expr};
use super::{value::Val, module::*, stack::*};
use crate::error::RuntimeError;

#[derive(Clone)]
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

impl FuncAddr {
    pub fn call(&self, params: Vec<Val>) -> Result<Vec<Val>,RuntimeError>{
        let mut stack = Stacks::new(&self, params);

        loop{
            stack.exec_instr()?;
            /*Reached Dummy Stack Frame*/
            if stack.activationFrameStack.len() == 1
            && stack.activationFrameStack.first().unwrap().labelStack.len() == 1
            && stack.activationFrameStack.first().unwrap().labelStack.first().unwrap().instrs.is_empty(){
                break;
            }
        }
        Ok(stack.activationFrameStack.pop().unwrap().labelStack.pop().unwrap().valueStack)
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
        *self.0.borrow_mut() = FuncInst::new(func, module);
    }
}

impl FuncInst{
    pub fn new(func: Func, module: Weak<ModuleInst>) -> FuncInst{
        FuncInst::RuntimeFunc{
            type_: module.upgrade().unwrap().types.get_by_idx(func.type_.clone()).clone(),
            module: module,
            code: func,
        }
    }
}