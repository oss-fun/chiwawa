//! Module instance containing runtime state (memories, tables, globals).

use super::value::*;
use super::{
    data::DataAddr, elem::ElemAddr, export::ExportInst, func::FuncAddr, global::GlobalAddr,
    mem::MemAddr, table::TableAddr,
};
use crate::error::RuntimeError;
use crate::shared::Shared;
use crate::structure::{instructions::*, module::*, types::*};
use crate::wasi::passthrough::PassthroughWasiImpl;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Instantiated module with all runtime components.
pub struct ModuleInst {
    pub types: Shared<Vec<FuncType>>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
    pub wasi_func_addrs: Vec<WasiFuncAddr>,
    pub wasi_impl: Option<Arc<PassthroughWasiImpl>>,
    /// Function named by the start section, run at instantiation.
    pub start_section: Option<FuncAddr>,
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

thread_local! {
    /// Instances whose segment initialization trapped.
    static TRAPPED: RefCell<Vec<Rc<ModuleInst>>> = const { RefCell::new(Vec::new()) };
}

fn elem_func_indices(elem: &Elem) -> Vec<i32> {
    elem.idxes
        .iter()
        .flatten()
        .map(|idx| idx.0 as i32)
        .collect()
}

fn limits_match(actual_size: u32, actual_max: Option<u32>, import: &Limits) -> bool {
    actual_size >= import.min
        && import
            .max
            .is_none_or(|max| actual_max.is_some_and(|actual| actual <= max))
}

impl ModuleInst {
    /// Instantiates a module with the given imports and command-line arguments.
    pub fn new(
        module: &Module,
        imports: ImportObjects,
        argv: Vec<String>,
    ) -> Result<Rc<ModuleInst>, RuntimeError> {
        Self::new_with_shared_memory(module, imports, argv, None)
    }

    /// Instantiates a module, substituting `shared_mem` for the memory the
    /// module defines itself, so wasi-threads instances share one buffer.
    pub fn new_with_shared_memory(
        module: &Module,
        imports: ImportObjects,
        argv: Vec<String>,
        mut shared_mem: Option<MemAddr>,
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
            start_section: None,
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
                let provided = imports
                    .get(&import.module.0)
                    .and_then(|module| module.get(&import.name.0))
                    .cloned();
                match &import.desc {
                    ImportDesc::Func(idx) => {
                        let val = provided.ok_or(RuntimeError::LinkError)?;
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
                    ImportDesc::Mem(mem_type) => {
                        let mem = provided
                            .and_then(Externval::as_mem)
                            .ok_or(RuntimeError::LinkError)?;
                        let matches = mem.is_shared() == mem_type.shared
                            && limits_match(
                                mem.mem_size() as u32,
                                mem.max_pages(),
                                &mem_type.limits,
                            );
                        if !matches {
                            return Err(RuntimeError::LinkError);
                        }
                        module_inst.mem_addrs.push(mem);
                    }
                    ImportDesc::Table(table_type) => {
                        let table = provided
                            .and_then(Externval::as_table)
                            .ok_or(RuntimeError::LinkError)?;
                        let actual = table.table_type();
                        let matches = actual.1 == table_type.1
                            && limits_match(table.len() as u32, actual.0.max, &table_type.0);
                        if !matches {
                            return Err(RuntimeError::LinkError);
                        }
                        module_inst.table_addrs.push(table);
                    }
                    ImportDesc::Global(global_type) => {
                        let global = provided
                            .and_then(Externval::as_global)
                            .ok_or(RuntimeError::LinkError)?;
                        if global.global_type() != *global_type {
                            return Err(RuntimeError::LinkError);
                        }
                        module_inst.global_addrs.push(global);
                    }
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
            let addr = match shared_mem.take_if(|_| mem.type_.shared) {
                Some(shared) => shared,
                None => MemAddr::new(&mem.type_),
            };
            module_inst.mem_addrs.push(addr);
        }
        if module_inst.mem_addrs.len() > 1 {
            return Err(RuntimeError::UnsupportedFeature("multiple memories"));
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
            let init = elem_func_indices(elem);
            module_inst
                .elem_addrs
                .push(ElemAddr::new(&elem.type_, &module_inst.func_addrs, &init));
        }

        for data in &module.datas {
            let init: Vec<u8> = data.init.iter().map(|x| x.0).collect();
            module_inst.data_addrs.push(DataAddr::new(&init));
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
        module_inst.start_section = module
            .start
            .as_ref()
            .map(|start| module_inst.func_addrs.get_by_idx(start.func).clone());

        let arc_module_inst = Rc::new(module_inst);

        for (base, func) in module.funcs.iter().enumerate() {
            let index = module.num_imported_funcs + base;
            arc_module_inst.func_addrs[index].replace(
                func.clone(),
                Rc::downgrade(&arc_module_inst),
                index as u32,
            );
        }

        // Segments go in after the functions have their bodies: a table shared with another instance may hand them out before this one is done.
        if let Err(e) = arc_module_inst.apply_active_segments(module) {
            TRAPPED.with(|trapped| trapped.borrow_mut().push(Rc::clone(&arc_module_inst)));
            return Err(e);
        }
        Ok(arc_module_inst)
    }

    fn apply_active_segments(&self, module: &Module) -> Result<(), RuntimeError> {
        for elem in module.elems.iter().filter(|e| e.mode == ElemMode::Active) {
            let offset = self.segment_offset(elem.offset.as_ref())?;
            let table_idx = elem.table_idx.map_or(0, |i| i.0);
            self.table_addrs[table_idx as usize].init(
                offset,
                &self.func_addrs,
                &elem_func_indices(elem),
            )?;
        }
        for data in module.datas.iter().filter(|d| d.mode == DataMode::Active) {
            let offset = self.segment_offset(data.offset.as_ref())?;
            let init: Vec<u8> = data.init.iter().map(|x| x.0).collect();
            let idx = data.memory.map_or(0, |i| i.0);
            self.mem_addrs[idx as usize].init(offset, &init)?;
        }
        Ok(())
    }

    fn segment_offset(&self, offset: Option<&Expr>) -> Result<usize, RuntimeError> {
        match offset {
            Some(expr) => {
                let value = self
                    .expr_to_const(expr)
                    .ok_or(RuntimeError::InvalidConstantExpression)?;
                Ok(value.to_i32()? as u32 as usize)
            }
            None => Ok(0),
        }
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
