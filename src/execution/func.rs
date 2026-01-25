//! Function instances and addresses.

use super::module::*;
use super::value::{Val, WasiFuncAddr};
use crate::error::RuntimeError;
use crate::structure::{module::*, types::*};
use std::cell::UnsafeCell;
use std::fmt::{self, Debug};
use std::rc::{Rc, Weak};

/// Reference-counted handle to a function instance.
/// Uses UnsafeCell for zero-cost access in the interpreter hot path.
/// Safety: WebAssembly execution is single-threaded and operations don't overlap.
#[derive(Clone)]
pub struct FuncAddr(Rc<UnsafeCell<FuncInst>>);

/// Function instance variants: runtime (Wasm), host, or WASI.
pub enum FuncInst {
    RuntimeFunc {
        type_: FuncType,
        module: Weak<ModuleInst>,
        code: Func,
    },
    HostFunc {
        type_: FuncType,
        host_code: Rc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError>>,
    },
    WasiFunc {
        type_: FuncType,
        wasi_func_addr: WasiFuncAddr,
    },
}

impl Debug for FuncAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Safety: Single-threaded access
        let inst = unsafe { &*self.0.get() };
        write!(f, "FuncAddr({:?})", inst)
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
    /// Allocates a placeholder function (replaced later during instantiation).
    pub fn alloc_empty() -> FuncAddr {
        FuncAddr(Rc::new(UnsafeCell::new(FuncInst::RuntimeFunc {
            type_: FuncType {
                params: Vec::new(),
                results: Vec::new(),
            },
            module: Weak::new(),
            code: Func {
                type_: TypeIdx(0),
                locals: Vec::new(),
                body: Rc::new(Vec::new()),
                reg_allocation: None,
                result_reg: None,
            },
        })))
    }

    /// Allocates a WASI function instance.
    pub fn alloc_wasi(wasi_func_addr: WasiFuncAddr) -> FuncAddr {
        let func_type = wasi_func_addr.func_type.to_func_type();
        FuncAddr(Rc::new(UnsafeCell::new(FuncInst::WasiFunc {
            type_: func_type,
            wasi_func_addr,
        })))
    }

    /// Replaces placeholder with actual function definition.
    pub fn replace(&self, func: Func, module: Weak<ModuleInst>) {
        let upgraded_module = module.upgrade().expect("Module weak ref expired");
        let func_type = upgraded_module.types.get_by_idx(func.type_).clone();
        drop(upgraded_module);

        let new_inst = FuncInst::RuntimeFunc {
            type_: func_type,
            module: module,
            code: func,
        };
        // Safety: Single-threaded access, no overlapping borrows
        unsafe {
            *self.0.get() = new_inst;
        }
    }

    /// Returns a reference to the function's type signature.
    /// Zero-copy access - no allocation.
    #[inline]
    pub fn func_type(&self) -> &FuncType {
        // Safety: Single-threaded access, no overlapping mutable access
        let inst = unsafe { &*self.0.get() };
        match inst {
            FuncInst::RuntimeFunc { type_, .. } => type_,
            FuncInst::HostFunc { type_, .. } => type_,
            FuncInst::WasiFunc { type_, .. } => type_,
        }
    }

    /// Extracts runtime function details if this is a Wasm function.
    pub fn get_runtime_func_details(&self) -> Option<(FuncType, Weak<ModuleInst>, Func)> {
        // Safety: Single-threaded access
        let inst = unsafe { &*self.0.get() };
        match inst {
            FuncInst::RuntimeFunc {
                type_,
                module,
                code,
            } => Some((type_.clone(), module.clone(), code.clone())),
            _ => None,
        }
    }

    /// Extracts host function details if this is a host function.
    pub fn get_host_func_details(
        &self,
    ) -> Option<(
        FuncType,
        Rc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError>>,
    )> {
        // Safety: Single-threaded access
        let inst = unsafe { &*self.0.get() };
        match inst {
            FuncInst::HostFunc { type_, host_code } => Some((type_.clone(), host_code.clone())),
            _ => None,
        }
    }

    /// Extracts WASI function details if this is a WASI function.
    pub fn get_wasi_func_details(&self) -> Option<(FuncType, WasiFuncAddr)> {
        // Safety: Single-threaded access
        let inst = unsafe { &*self.0.get() };
        match inst {
            FuncInst::WasiFunc {
                type_,
                wasi_func_addr,
            } => Some((type_.clone(), wasi_func_addr.clone())),
            _ => None,
        }
    }

    /// Returns a reference to the underlying function instance.
    /// # Safety
    /// Caller must ensure no mutable access occurs during the lifetime of the reference.
    #[inline]
    pub fn read_lock(&self) -> &FuncInst {
        // Safety: Single-threaded access, caller ensures no mutable access
        unsafe { &*self.0.get() }
    }

    /// Returns a reference to the inner Rc.
    pub fn get_rc(&self) -> &Rc<UnsafeCell<FuncInst>> {
        &self.0
    }
}
