//! Module instance containing runtime state (memories, tables, globals).

use super::value::*;
use super::{
    data::DataAddr, elem::ElemAddr, export::ExportInst, func::FuncAddr, global::GlobalAddr,
    mem::MemAddr, table::TableAddr,
};
use crate::error::RuntimeError;
use crate::structure::{instructions::*, module::*, types::*};
use crate::wasi::passthrough::PassthroughWasiImpl;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Instantiated module with all runtime components.
pub struct ModuleInst {
    pub types: Rc<Vec<FuncType>>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
    pub wasi_func_addrs: Vec<WasiFuncAddr>,
    pub wasi_impl: Option<Arc<PassthroughWasiImpl>>,
}

/// Trait for indexed access to instance vectors.
pub trait GetInstanceByIdx<Idx>
where
    Idx: GetIdx,
    Self: std::ops::Index<usize>,
{
    /// Gets instance by typed index.
    fn get_by_idx(&self, idx: Idx) -> &Self::Output {
        &self[idx.to_usize()]
    }
}

impl GetInstanceByIdx<TypeIdx> for Vec<FuncType> {}
impl GetInstanceByIdx<FuncIdx> for Vec<FuncAddr> {}
impl GetInstanceByIdx<TableIdx> for Vec<TableAddr> {}
impl GetInstanceByIdx<MemIdx> for Vec<MemAddr> {}
impl GetInstanceByIdx<GlobalIdx> for Vec<GlobalAddr> {}

/// Map of module name -> (export name -> external value) for imports.
pub type ImportObjects = FxHashMap<String, FxHashMap<String, Externval>>;

impl ModuleInst {
    /// Instantiates a module with the given imports and command-line arguments.
    pub fn new(
        module: &Module,
        imports: ImportObjects,
        argv: Vec<String>,
    ) -> Result<Rc<ModuleInst>, RuntimeError> {
        let mut module_inst = ModuleInst {
            types: module.types.clone(),
            func_addrs: Vec::new(),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs: Vec::new(),
            elem_addrs: Vec::new(),
            data_addrs: Vec::new(),
            exports: Vec::new(),
            wasi_func_addrs: Vec::new(),
            wasi_impl: None,
        };

        // Check if we need WASI support
        let needs_wasi = module
            .imports
            .iter()
            .any(|import| matches!(import.desc, ImportDesc::WasiFunc(_)));

        if needs_wasi {
            module_inst.wasi_impl = Some(Arc::new(PassthroughWasiImpl::new(argv)));
        }

        /*Import processing*/
        if module.imports.len() != 0 {
            for import in &module.imports {
                match &import.desc {
                    ImportDesc::Func(idx) => {
                        let val = imports
                            .get(&import.module.0)
                            .and_then(|module| module.get(&import.name.0))
                            .cloned()
                            .ok_or_else(|| RuntimeError::LinkError)?;
                        module_inst.func_addrs.push(
                            val.as_func()
                                .filter(|func| {
                                    func.func_type()
                                        .type_match(&module_inst.types[idx.0 as usize])
                                })
                                .ok_or_else(|| RuntimeError::LinkError)?,
                        );
                    }
                    ImportDesc::WasiFunc(wasi_func_type) => {
                        // Create WASI function address
                        let wasi_func_addr = WasiFuncAddr::new(wasi_func_type.clone());
                        module_inst
                            .func_addrs
                            .push(FuncAddr::alloc_wasi(wasi_func_addr.clone()));
                        module_inst.wasi_func_addrs.push(wasi_func_addr);
                    }
                    _ => todo!(),
                }
            }
        }

        for _ in &module.funcs {
            module_inst.func_addrs.push(FuncAddr::alloc_empty())
        }

        for table in &module.tables {
            module_inst.table_addrs.push(TableAddr::new(&table.type_))
        }

        for mem in &module.mems {
            module_inst.mem_addrs.push(MemAddr::new(&mem.type_))
        }

        for global in &module.globals {
            match module_inst.expr_to_const(&global.init) {
                Some(v) => module_inst
                    .global_addrs
                    .push(GlobalAddr::new(&global.type_, v)),
                None => return Err(RuntimeError::InstantiateFailed),
            }
        }

        for elem in &module.elems {
            if elem.type_ == RefType::ExternalRef {
                panic!();
            }
            let mut init: Vec<i32> = Vec::new();
            if let Some(idxes) = &elem.idxes {
                for idx in idxes {
                    init.push(idx.0 as i32);
                }
            }
            module_inst
                .elem_addrs
                .push(ElemAddr::new(&elem.type_, &module_inst.func_addrs, &init));

            if elem.mode == ElemMode::Active {
                let offset_res = match &elem.offset {
                    Some(x) => module_inst
                        .expr_to_const(x)
                        .ok_or(RuntimeError::InvalidConstantExpression)?
                        .to_i32(),
                    None => Ok(0),
                };
                let offset = offset_res?;

                let table_idx = match &elem.table_idx {
                    Some(i) => i.0,
                    None => 0,
                };
                module_inst.table_addrs[table_idx as usize].init(
                    offset as usize,
                    &module_inst.func_addrs,
                    &init,
                );
            }
        }

        for data in &module.datas {
            let init: Vec<u8> = data.init.iter().map(|x| x.0).collect();
            module_inst.data_addrs.push(DataAddr::new(&init));

            if data.mode == DataMode::Active {
                let offset_res = match &data.offset {
                    Some(x) => module_inst
                        .expr_to_const(x)
                        .ok_or(RuntimeError::InvalidConstantExpression)?
                        .to_i32(),
                    None => Ok(0),
                };
                let offset = offset_res?;

                let idx = match &data.memory {
                    Some(i) => i.0,
                    None => 0,
                };

                module_inst.mem_addrs[idx as usize].init(offset as usize, &init);
            }
        }

        for export in &module.exports {
            module_inst.exports.push(ExportInst {
                name: export.name.0.clone(),
                value: match &export.desc {
                    ExportDesc::Func(idx) => {
                        Externval::Func(module_inst.func_addrs[idx.0 as usize].clone())
                    }
                    ExportDesc::Table(idx) => {
                        Externval::Table(module_inst.table_addrs[idx.0 as usize].clone())
                    }
                    ExportDesc::Mem(idx) => {
                        Externval::Mem(module_inst.mem_addrs[idx.0 as usize].clone())
                    }
                    ExportDesc::Global(idx) => {
                        Externval::Global(module_inst.global_addrs[idx.0 as usize].clone())
                    }
                },
            })
        }
        let arc_module_inst = Rc::new(module_inst);

        for (base, func) in module.funcs.iter().enumerate() {
            let index = base
                + module
                    .imports
                    .iter()
                    .map(|i| match i.desc {
                        ImportDesc::Func(_) => 1,
                        ImportDesc::WasiFunc(_) => 1,
                        _ => 0,
                    })
                    .sum::<usize>();
            arc_module_inst.func_addrs[index]
                .replace(func.clone(), Rc::downgrade(&arc_module_inst));
        }
        if let Some(start) = &module.start {
            arc_module_inst.func_addrs.get_by_idx(start.func);
        }
        Ok(arc_module_inst)
    }

    /// Looks up an exported function by name.
    pub fn get_export_func(&self, name: &str) -> Result<FuncAddr, RuntimeError> {
        let externval = self
            .exports
            .iter()
            .find(|export| export.name == name)
            .map(|export| export.value.clone());
        if let Some(Externval::Func(x)) = externval {
            Ok(x)
        } else {
            Err(RuntimeError::ExportFuncNotFound)
        }
    }

    /// Evaluates a constant expression to a value.
    fn expr_to_const(&self, expr: &Expr) -> Option<Val> {
        match &expr.0[..] {
            &[Instr::I32Const(i)] => Some(Val::Num(Num::I32(i))),
            &[Instr::I64Const(i)] => Some(Val::Num(Num::I64(i))),
            &[Instr::F32Const(i)] => Some(Val::Num(Num::F32(i as f32))),
            &[Instr::F64Const(i)] => Some(Val::Num(Num::F64(i as f64))),
            &[Instr::V128Const(i)] => Some(Val::Vec_(Vec_::V128(i))),
            &[Instr::RefNull(_)] => Some(Val::Ref(Ref::RefNull)),
            [Instr::GlobalGet(i)] => Some(self.global_addrs.get_by_idx(*i).get()),
            _ => None,
        }
    }
}
