use super::module::*;
use super::value::{Val, WasiFuncAddr};
use crate::error::RuntimeError;
use crate::structure::{module::*, types::*};
use std::sync::{Arc, RwLock, Weak as SyncWeak};
use std::{
    fmt::{self, Debug},
    sync::RwLockReadGuard,
};

#[derive(Clone)]
pub struct FuncAddr(Arc<RwLock<FuncInst>>);

pub enum FuncInst {
    RuntimeFunc {
        type_: FuncType,
        module: SyncWeak<ModuleInst>,
        code: Func,
    },
    HostFunc {
        type_: FuncType,
        host_code: Arc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError> + Send + Sync>,
    },
    WasiFunc {
        type_: FuncType,
        wasi_func_addr: WasiFuncAddr,
    },
}

impl Debug for FuncAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.try_read() {
            Ok(guard) => write!(f, "FuncAddr({:?})", *guard),
            Err(_) => write!(f, "FuncAddr(<Locked>)"),
        }
    }
}

impl Debug for FuncInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncInst::RuntimeFunc {
                type_,
                module,
                code,
            } => f
                .debug_struct("RuntimeFunc")
                .field("type_", type_)
                .field("module", &module.upgrade().is_some())
                .field("code", code)
                .finish(),
            FuncInst::HostFunc {
                type_,
                host_code: _,
            } => f
                .debug_struct("HostFunc")
                .field("type_", type_)
                .field("host_code", &"<host_code>")
                .finish(),
            FuncInst::WasiFunc {
                type_,
                wasi_func_addr,
            } => f
                .debug_struct("WasiFunc")
                .field("type_", type_)
                .field("wasi_func_addr", wasi_func_addr)
                .finish(),
        }
    }
}

impl FuncAddr {
    pub fn alloc_empty() -> FuncAddr {
        FuncAddr(Arc::new(RwLock::new(FuncInst::RuntimeFunc {
            type_: FuncType {
                params: Vec::new(),
                results: Vec::new(),
            },
            module: SyncWeak::new(),
            code: Func {
                type_: TypeIdx(0),
                locals: Vec::new(),
                body: Vec::new(),
            },
        })))
    }

    pub fn alloc_wasi(wasi_func_addr: WasiFuncAddr) -> FuncAddr {
        let func_type = wasi_func_addr.func_type.to_func_type();
        FuncAddr(Arc::new(RwLock::new(FuncInst::WasiFunc {
            type_: func_type,
            wasi_func_addr,
        })))
    }

    pub fn replace(&self, func: Func, module: SyncWeak<ModuleInst>) {
        let upgraded_module = module.upgrade().expect("Module weak ref expired");
        let func_type = upgraded_module.types.get_by_idx(func.type_.clone()).clone();
        drop(upgraded_module);

        let new_inst = FuncInst::RuntimeFunc {
            type_: func_type,
            module: module,
            code: func,
        };
        *self.0.write().expect("RwLock poisoned") = new_inst;
    }

    pub fn func_type(&self) -> FuncType {
        match &*self.0.read().expect("RwLock poisoned") {
            FuncInst::RuntimeFunc { type_, .. } => type_.clone(),
            FuncInst::HostFunc { type_, .. } => type_.clone(),
            FuncInst::WasiFunc { type_, .. } => type_.clone(),
        }
    }

    pub fn get_runtime_func_details(&self) -> Option<(FuncType, SyncWeak<ModuleInst>, Func)> {
        match &*self.0.read().expect("RwLock poisoned") {
            FuncInst::RuntimeFunc {
                type_,
                module,
                code,
            } => Some((type_.clone(), module.clone(), code.clone())),
            _ => None,
        }
    }

    pub fn get_host_func_details(
        &self,
    ) -> Option<(
        FuncType,
        Arc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError> + Send + Sync>,
    )> {
        match &*self.0.read().expect("RwLock poisoned") {
            FuncInst::HostFunc { type_, host_code } => Some((type_.clone(), host_code.clone())),
            _ => None,
        }
    }

    pub fn get_wasi_func_details(&self) -> Option<(FuncType, WasiFuncAddr)> {
        match &*self.0.read().expect("RwLock poisoned") {
            FuncInst::WasiFunc {
                type_,
                wasi_func_addr,
            } => Some((type_.clone(), wasi_func_addr.clone())),
            _ => None,
        }
    }

    pub fn read_lock(
        &self,
    ) -> Result<RwLockReadGuard<FuncInst>, std::sync::PoisonError<RwLockReadGuard<'_, FuncInst>>>
    {
        self.0.read()
    }

    pub fn get_arc(&self) -> &Arc<RwLock<FuncInst>> {
        &self.0
    }
}
