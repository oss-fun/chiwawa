use std::{rc::Rc, cell::RefCell, rc::Weak};
use crate::structure::{types::*,module::*};
use super::{value::Val, module::ModuleInst};
use crate::error::RuntimeError;

pub struct FuncAddr(Rc<RefCell<FuncInst>>);
pub enum FuncInst {
    RuntimeFunc{
        ptype_: FuncType,
        module: Weak<ModuleInst>,
        code: Func,
    },
    HostFunc{
        type_: FuncType,
        host_code: Rc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError>>,
    },
}