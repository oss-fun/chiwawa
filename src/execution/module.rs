use std::rc::Rc;
use std::collections::HashMap;
use crate::structure::{types::*, module::*, instructions::*};
use crate::error::RuntimeError;
use super::value::*;
use super::{func::FuncAddr, table::TableAddr, mem::MemAddr, global::GlobalAddr, elem::ElemAddr, data::DataAddr, export::ExportInst};

pub struct Results(Option<Vec<Val>>);

pub struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
}

pub trait GetInstanceByIdx<Idx>
where
Idx: GetIdx,
Self: std::ops::Index<usize>,
{
    fn get_by_idx(&self, idx: Idx) -> &Self::Output{
        &self[idx.to_usize()]
    }
}

impl GetInstanceByIdx<FuncIdx> for Vec<FuncAddr>{}
impl GetInstanceByIdx<TableIdx> for Vec<TableAddr>{}
impl GetInstanceByIdx<MemIdx> for Vec<MemAddr>{}
impl GetInstanceByIdx<GlobalIdx> for Vec<GlobalAddr>{}

pub type ImportObjects = HashMap<String, HashMap<String, Externval>>;

impl ModuleInst {
    pub fn new(module: &Module, imports: ImportObjects) -> Result<Rc<ModuleInst>, RuntimeError>{
        let mut module_inst = ModuleInst {
            types: module.types.clone(),
            func_addrs: Vec::new(),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs:Vec::new(),
            elem_addrs:Vec::new(),
            data_addrs:Vec::new(),
            exports:Vec::new(),
        };

        /*Import is not supported*/
        if module.imports.len() != 0{
            todo!();
        }

        for func in &module.funcs{
            module_inst.func_addrs.push(FuncAddr::alloc_empty())
        }

        for table in &module.tables {
            module_inst.table_addrs.push(TableAddr::new(&table.type_))
        }

        for mem in &module.mems {
            module_inst.mem_addrs.push(MemAddr::new(&mem.type_))
        }

        for global in &module.globals {
            module_inst.global_addrs.push(GlobalAddr::new(&global.type_, ModuleInst::expr_to_const(&global.init)));
        }

        for elem in &module.elems{
            if elem.type_ == RefType::ExternalRef{
                panic!();
            }
            let init: Vec<i32> = elem.init.clone().into_iter().map(|expr| ModuleInst::expr_to_const(&expr).to_i32()).collect();

            module_inst.elem_addrs.push(ElemAddr::new(&elem.type_, &module_inst.func_addrs, &init));

            if elem.mode == ElemMode::Active{
                let offset = match &elem.offset {
                    Some(x) => ModuleInst::expr_to_const(x).to_i32(),
                    None => 0,
                };
    
                let table_idx = match &elem.table_idx{
                    Some(i) => i.0,
                    None => 0,
                };
                module_inst.table_addrs[table_idx as usize].init(offset as usize, &module_inst.func_addrs, &init);
            }
        }

        for data in &module.datas{
            let init: Vec<u8> = data.init.iter().map(|x|x.0).collect();
            module_inst.data_addrs.push(DataAddr::new(&init));
            let offset = match &data.offset {
                Some(x) => ModuleInst::expr_to_const(x).to_i32(),
                None => 0,
            };

            let idx = match &data.memory{
                Some(i) => i.0,
                None => 0,
            };

            module_inst.mem_addrs[idx as usize].init(offset as usize, &init);
        }

        for export in &module.exports {
            module_inst.exports.push(
                ExportInst{
                    name: export.name.0.clone(),
                    value: match &export.desc {
                        ExportDesc::Func(idx) => Externval::Func(module_inst.func_addrs.get_by_idx(idx.clone()).clone()),
                        ExportDesc::Table(idx) => Externval::Table(module_inst.table_addrs.get_by_idx(idx.clone()).clone()),
                        ExportDesc::Mem(idx) => Externval::Mem(module_inst.mem_addrs.get_by_idx(idx.clone()).clone()),
                        ExportDesc::Global(idx) => Externval::Global(module_inst.global_addrs.get_by_idx(idx.clone()).clone()),
                    },
                }
            )
        }

        Ok(Rc::new(module_inst))
    }
    fn expr_to_const(expr: &Expr) -> Val{
        match &expr.0[..] {
            &[Instr::I32Const(i)] => Val::Num(Num::I32(i)),
            &[Instr::I64Const(i)] => Val::Num(Num::I64(i)),
            &[Instr::F32Const(i)] => Val::Num(Num::F32(i)),
            &[Instr::F64Const(i)] => Val::Num(Num::F64(i)),
            &[Instr::V128Const(i)] => Val::Vec_(Vec_::V128(i)),
            _ => todo!(),
        }
    }
}