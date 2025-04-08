use wasmparser::{Parser, Payload::*, TypeRef, ValType, SectionLimited, ExternalKind, FunctionBody, OperatorsIteratorWithOffsets};
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;

use crate::structure::module::*;
use crate::structure::types::*;
use crate::structure::instructions::*;
use crate::error::ParserError;

fn match_value_type(t: ValType) -> ValueType {
    match t {
        ValType::I32 => ValueType::NumType(NumType::I32),
        ValType::I64 => ValueType::NumType(NumType::I64),
        ValType::F32 => ValueType::NumType(NumType::F32),
        ValType::F64 => ValueType::NumType(NumType::F64),
        ValType::V128 => ValueType::VecType(VecType::V128),
        ValType::Ref(ref_type) => {
            if ref_type.is_func_ref() {
                ValueType::RefType(RefType::FuncRef)
            } else {
                ValueType::RefType(RefType::ExternalRef)
            }
    }}
}

fn types_to_vec(types: &[ValType], vec: &mut Vec<ValueType>) {
    for t in types.iter(){
        vec.push(match_value_type(*t));
    }
}

fn decode_type_section(body: SectionLimited<'_, wasmparser::RecGroup>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for functype in body.into_iter_err_on_gc_types() {
        let functype = functype?;

        let mut params = Vec::new();
        let mut results = Vec::new();
        types_to_vec(functype.params(), &mut params);
        types_to_vec(functype.results(), &mut results);

        module.types.push(FuncType{
                params,
                results
        });
    };
    Ok(())
}

fn decode_func_section(body: SectionLimited<'_, u32>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for func in body{
        let index = func?;
        let typeidx = TypeIdx(index);
        module.funcs.push(Func{
            type_: typeidx,
            locals: Vec::new(),
            body: Expr(Vec::new()),
        });
    }

    Ok(())
}

fn decode_import_section(body: SectionLimited<'_, wasmparser::Import<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for import in body {
        let import = import?;
        let desc = match import.ty {
            TypeRef::Func(type_index) => {
                module.num_imported_funcs += 1;
                ImportDesc::Func(TypeIdx(type_index))
            },
            TypeRef::Table(table_type) => {
                let max = match table_type.maximum{
                    Some(m) =>  Some(TryFrom::try_from(m).unwrap()),
                    None => None

                };
                let limits = Limits{min: TryFrom::try_from(table_type.initial).unwrap(), max};
                let reftype = if table_type.element_type.is_func_ref() {
                    RefType::FuncRef
                } else {
                    RefType::ExternalRef
                };

                ImportDesc::Table(TableType(limits,reftype))
            },
            TypeRef::Memory(memory) => {
                let max = match memory.maximum{
                    Some(m) =>  Some(TryFrom::try_from(m).unwrap()),
                    None => None

                };
                let limits = Limits{min: TryFrom::try_from(memory.initial).unwrap(), max};
                ImportDesc::Mem(MemType(limits))
            },
            TypeRef::Global(global) => {
                let mut_ = if global.mutable{
                    Mut::Var
                } else {
                    Mut::Const
                };
                let value_type = match_value_type(global.content_type);
                ImportDesc::Global(GlobalType(mut_,value_type))
            },
            TypeRef::Tag(_) => todo!()
        };
        module.imports.push(
            Import{
                module: Name(import.module.to_string()),
                name: Name(import.name.to_string()),
                desc,
            }
        );
    }
    Ok(())
}

fn decode_export_section(body: SectionLimited<'_, wasmparser::Export<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for export in body {
        let export = export?;
        let index = export.index;
        let desc = match export.kind {
            ExternalKind::Func => {
                ExportDesc::Func(FuncIdx(index))
            },
            ExternalKind::Table => {
                ExportDesc::Table(TableIdx(index))
            },
            ExternalKind::Memory => {
                ExportDesc::Mem(MemIdx(index))
            },
            ExternalKind::Global => {
                ExportDesc::Global(GlobalIdx(index))
            },
            ExternalKind::Tag => todo!()
        };
        module.exports.push(
            Export{
                name: Name(export.name.to_string()),
                desc,
            }
        );
    };
    Ok(())
}

fn decode_mem_section(body: SectionLimited<'_, wasmparser::MemoryType>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for memory in body {
        let memory = memory?;
        let max = match memory.maximum {
            Some(m) => Some(TryFrom::try_from(m).unwrap()),
            None => None
        };
        let limits = Limits{min: TryFrom::try_from(memory.initial).unwrap(), max};
        module.mems.push(Mem{
            type_: MemType(limits)
        });
    }
    Ok(())
}

fn decode_table_section(body: SectionLimited<'_, wasmparser::Table<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for table in body{
        let table = table?;
        let table_type = table.ty;

        let max = match table_type.maximum{
            Some(m) =>  Some(TryFrom::try_from(m).unwrap()),
            None => None

        };
        let limits = Limits{min: TryFrom::try_from(table_type.initial).unwrap(), max};

        let reftype = if table_type.element_type.is_func_ref() {
            RefType::FuncRef
        } else {
            RefType::ExternalRef
        };
        module.tables.push(Table{
            type_: TableType(limits,reftype)
        });
    }
    Ok(())
}

fn decode_global_section(body: SectionLimited<'_, wasmparser::Global<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>>{
    for global in body {
        let global = global?;
        let mut_ = if global.ty.mutable{
            Mut::Var
        } else {
            Mut::Const
        };
        let value_type = match_value_type(global.ty.content_type);
        let type_  = GlobalType(mut_,value_type);
        let init = parse_initexpr(global.init_expr)?;
        module.globals.push(
            Global{type_, init}
        );

    }
    Ok(())
}

fn parse_initexpr(expr: wasmparser::ConstExpr<'_>) -> Result<Expr, Box<dyn std::error::Error>>{
    let mut instrs = Vec::new();
    let mut ops = expr.get_operators_reader().into_iter_with_offsets().peekable();
    while let Some(res) = ops.next() {
        let (op, offset) = res?;

        if (matches!(op,wasmparser::Operator::End) && ops.peek().is_none()) {
            break;
        }

        match op {
            wasmparser::Operator::I32Const {value} => instrs.push(Instr::I32Const(value)),
            wasmparser::Operator::I64Const {value} => instrs.push(Instr::I64Const(value)),
            wasmparser::Operator::F32Const {value} => instrs.push(Instr::F32Const(f32::from_bits(value.bits()))),
            wasmparser::Operator::F64Const {value} => instrs.push(Instr::F64Const(f64::from_bits(value.bits()))),
            wasmparser::Operator::RefNull {..} => instrs.push(Instr::RefNull(RefType::ExternalRef)),
            wasmparser::Operator::RefFunc {function_index} => instrs.push(Instr::RefFunc(FuncIdx(function_index))),
            wasmparser::Operator::GlobalGet {global_index} => instrs.push(Instr::GlobalGet(GlobalIdx(global_index))),

            _ => return Err(Box::new(ParserError::InitExprUnsupportedOPCodeError{offset})),
        }
    }
    Ok(Expr(instrs))
}

fn decode_elem_section(body: SectionLimited<'_, wasmparser::Element<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>> {
    for (_index, entry) in body.into_iter().enumerate() {
        let entry = entry?;
        let _cnt = 0; // Marked as unused
        let (type_, init, idxes) = match entry.items {
            wasmparser::ElementItems::Functions(funcs) => {
                let mut idxes = Vec::new();
                for func in funcs {
                    idxes.push(FuncIdx(func?));
                }
                (RefType::FuncRef, None, Some(idxes))
            },
            wasmparser::ElementItems::Expressions(type_, items) => {
                let mut exprs = Vec::new();
                for expr in items {
                    let expr = parse_initexpr(expr?)?;
                    exprs.push(expr);
                }

                if type_.is_func_ref() {
                    (RefType::FuncRef, Some(exprs), None)
                } else {
                    (RefType::ExternalRef, Some(exprs),None)
                }
            }
        };
        let (mode, table_idx, offset) = match entry.kind {
            wasmparser::ElementKind::Active{table_index, offset_expr} => {
                let expr = parse_initexpr(offset_expr)?;
                let table_index = table_index.unwrap_or(0);
                (ElemMode::Active, Some(TableIdx(table_index)), Some(expr))
            },
            wasmparser::ElementKind::Passive => {
                (ElemMode::Passive, None, None)

            },
            wasmparser::ElementKind::Declared => {
                (ElemMode::Declarative, None, None)
            }
        };
        module.elems.push(Elem{
            type_,
            init,
            idxes,
            mode,
            table_idx,
            offset,
        });
    }
    Ok(())
}
fn decode_data_section(body: SectionLimited<'_, wasmparser::Data<'_>>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>> {
    for (_index, entry) in body.into_iter().enumerate() {
        let entry = entry?;
        let init = entry.data.iter().map(|x| Byte(*x)).collect::<Vec<Byte>>();
        let (mode, memory, offset) = match entry.kind {
            wasmparser::DataKind::Passive => {
                (DataMode::Passive, None, None)
            },
            wasmparser::DataKind::Active{memory_index, offset_expr} => {
                let expr = parse_initexpr(offset_expr)?;
                (DataMode::Active, Some(MemIdx(memory_index)), Some(expr))
            },
        };

        module.datas.push(Data{
            init,
            mode,
            memory,
            offset,
        })
    }
    Ok(())
}

fn decode_code_section(body: FunctionBody<'_>, module: &mut Module) -> Result<(), Box<dyn std::error::Error>> {
    for pair in body.get_locals_reader()? {
        let (cnt, ty) = pair?;
        let ty = match_value_type(ty);
        module.funcs[module.code_index].locals.push((cnt,ty));
    }

    // --- Reverted and Modified Code ---
    let ops_reader = body.get_operators_reader()?;
    let mut ops_iter = ops_reader.into_iter_with_offsets().peekable();
    let instrs = decode_instrs_with_markers(&mut ops_iter)?; // Use new function
    module.funcs[module.code_index].body = Expr(instrs);
    module.code_index += 1;
    // --- End Reverted and Modified Code ---
    Ok(())
}

// Modified function to generate flat list with markers
fn decode_instrs_with_markers(ops: &mut Peekable<OperatorsIteratorWithOffsets<'_>>) -> Result<Vec<Instr>, Box<dyn std::error::Error>> {
    let mut instrs = Vec::new();
    // Loop until the iterator is exhausted
    while let Some(res) = ops.next() {
        let (op, offset) = res?;
        // map_operator_to_instr handles all operators now, including markers
        instrs.push(map_operator_to_instr(op, offset)?);
    }
    Ok(instrs)
}


// Renamed and modified to generate flat instructions and markers
fn map_operator_to_instr(op: wasmparser::Operator, _offset: usize) -> Result<Instr, Box<dyn std::error::Error>> {
    let instr = match op {
        /* Numeric Instructions */
        wasmparser::Operator::I32Const { value } => Instr::I32Const(value),
        wasmparser::Operator::I64Const {value} => Instr::I64Const(value),
        wasmparser::Operator::F32Const {value} => Instr::F32Const(f32::from_bits(value.bits())),
        wasmparser::Operator::F64Const {value} => Instr::F64Const(f64::from_bits(value.bits())),
        wasmparser::Operator::I32Clz => Instr::I32Clz,
        wasmparser::Operator::I32Ctz => Instr::I32Ctz,
        wasmparser::Operator::I32Popcnt => Instr::I32Popcnt,
        wasmparser::Operator::I64Clz => Instr::I64Clz,
        wasmparser::Operator::I64Ctz => Instr::I64Ctz,
        wasmparser::Operator::I64Popcnt => Instr::I64Popcnt,
        wasmparser::Operator::F32Abs => Instr::F32Abs,
        wasmparser::Operator::F32Neg => Instr::F32Neg,
        wasmparser::Operator::F32Sqrt => Instr::F32Sqrt,
        wasmparser::Operator::F32Ceil => Instr::F32Ceil,
        wasmparser::Operator::F32Floor => Instr::F32Floor,
        wasmparser::Operator::F32Trunc => Instr::F32Trunc,
        wasmparser::Operator::F32Nearest => Instr::F32Nearest,
        wasmparser::Operator::F64Abs => Instr::F64Abs,
        wasmparser::Operator::F64Neg => Instr::F64Neg,
        wasmparser::Operator::F64Sqrt => Instr::F64Sqrt,
        wasmparser::Operator::F64Ceil => Instr::F64Ceil,
        wasmparser::Operator::F64Floor => Instr::F64Floor,
        wasmparser::Operator::F64Trunc => Instr::F64Trunc,
        wasmparser::Operator::F64Nearest => Instr::F64Nearest,
        wasmparser::Operator::I32Add => Instr::I32Add,
        wasmparser::Operator::I32Sub => Instr::I32Sub,
        wasmparser::Operator::I32Mul => Instr::I32Mul,
        wasmparser::Operator::I32DivS => Instr::I32DivS,
        wasmparser::Operator::I32DivU => Instr::I32DivU,
        wasmparser::Operator::I32RemS => Instr::I32RemS,
        wasmparser::Operator::I32RemU => Instr::I32RemU,
        wasmparser::Operator::I32And => Instr::I32And,
        wasmparser::Operator::I32Or => Instr::I32Or,
        wasmparser::Operator::I32Xor => Instr::I32Xor,
        wasmparser::Operator::I32Shl => Instr::I32Shl,
        wasmparser::Operator::I32ShrS => Instr::I32ShrS,
        wasmparser::Operator::I32ShrU => Instr::I32ShrU,
        wasmparser::Operator::I32Rotl => Instr::I32Rotl,
        wasmparser::Operator::I32Rotr => Instr::I32Rotr,
        wasmparser::Operator::I64Add => Instr::I64Add,
        wasmparser::Operator::I64Sub => Instr::I64Sub,
        wasmparser::Operator::I64Mul => Instr::I64Mul,
        wasmparser::Operator::I64DivS => Instr::I64DivS,
        wasmparser::Operator::I64DivU => Instr::I64DivU,
        wasmparser::Operator::I64RemS => Instr::I64RemS,
        wasmparser::Operator::I64RemU => Instr::I64RemU,
        wasmparser::Operator::I64And => Instr::I64And,
        wasmparser::Operator::I64Or => Instr::I64Or,
        wasmparser::Operator::I64Xor => Instr::I64Xor,
        wasmparser::Operator::I64Shl => Instr::I64Shl,
        wasmparser::Operator::I64ShrS => Instr::I64ShrS,
        wasmparser::Operator::I64ShrU => Instr::I64ShrU,
        wasmparser::Operator::I64Rotl => Instr::I64Rotl,
        wasmparser::Operator::I64Rotr => Instr::I64Rotr,
        wasmparser::Operator::F32Add => Instr::F32Add,
        wasmparser::Operator::F32Sub => Instr::F32Sub,
        wasmparser::Operator::F32Mul => Instr::F32Mul,
        wasmparser::Operator::F32Div => Instr::F32Div,
        wasmparser::Operator::F32Min => Instr::F32Min,
        wasmparser::Operator::F32Max => Instr::F32Max,
        wasmparser::Operator::F32Copysign => Instr::F32Copysign,
        wasmparser::Operator::F64Add => Instr::F64Add,
        wasmparser::Operator::F64Sub => Instr::F64Sub,
        wasmparser::Operator::F64Mul => Instr::F64Mul,
        wasmparser::Operator::F64Div => Instr::F64Div,
        wasmparser::Operator::F64Min => Instr::F64Min,
        wasmparser::Operator::F64Max => Instr::F64Max,
        wasmparser::Operator::F64Copysign => Instr::F64Copysign,
        wasmparser::Operator::I32Eqz => Instr::I32Eqz,
        wasmparser::Operator::I64Eqz => Instr::I64Eqz,
        wasmparser::Operator::I32Eq => Instr::I32Eq,
        wasmparser::Operator::I32Ne => Instr::I32Ne,
        wasmparser::Operator::I32LtS => Instr::I32LtS,
        wasmparser::Operator::I32LtU => Instr::I32LtU,
        wasmparser::Operator::I32GtS => Instr::I32GtS,
        wasmparser::Operator::I32GtU => Instr::I32GtU,
        wasmparser::Operator::I32LeS => Instr::I32LeS,
        wasmparser::Operator::I32LeU => Instr::I32LeU,
        wasmparser::Operator::I32GeS => Instr::I32GeS,
        wasmparser::Operator::I32GeU => Instr::I32GeU,
        wasmparser::Operator::I64Eq => Instr::I64Eq,
        wasmparser::Operator::I64Ne => Instr::I64Ne,
        wasmparser::Operator::I64LtS => Instr::I64LtS,
        wasmparser::Operator::I64LtU => Instr::I64LtU,
        wasmparser::Operator::I64GtS => Instr::I64GtS,
        wasmparser::Operator::I64GtU => Instr::I64GtU,
        wasmparser::Operator::I64LeS => Instr::I64LeS,
        wasmparser::Operator::I64LeU => Instr::I64LeU,
        wasmparser::Operator::I64GeS => Instr::I64GeS,
        wasmparser::Operator::I64GeU => Instr::I64GeU,
        wasmparser::Operator::F32Eq => Instr::F32Eq,
        wasmparser::Operator::F32Ne => Instr::F32Ne,
        wasmparser::Operator::F32Lt => Instr::F32Lt,
        wasmparser::Operator::F32Gt => Instr::F32Gt,
        wasmparser::Operator::F32Le => Instr::F32Le,
        wasmparser::Operator::F32Ge => Instr::F32Ge,
        wasmparser::Operator::F64Eq => Instr::F64Eq,
        wasmparser::Operator::F64Ne => Instr::F64Ne,
        wasmparser::Operator::F64Lt => Instr::F64Lt,
        wasmparser::Operator::F64Gt => Instr::F64Gt,
        wasmparser::Operator::F64Le => Instr::F64Le,
        wasmparser::Operator::F64Ge => Instr::F64Ge,
        wasmparser::Operator::I32WrapI64 => Instr::I32WrapI64,
        wasmparser::Operator::I64ExtendI32U => Instr::I64ExtendI32U,
        wasmparser::Operator::I64ExtendI32S => Instr::I64ExtendI32S,
        wasmparser::Operator::I32TruncF32S => Instr::I32TruncF32S,
        wasmparser::Operator::I32TruncF32U => Instr::I32TruncF32U,
        wasmparser::Operator::I32TruncF64S => Instr::I32TruncF64S,
        wasmparser::Operator::I32TruncF64U => Instr::I32TruncF64U,
        wasmparser::Operator::I64TruncF32S => Instr::I64TruncF32S,
        wasmparser::Operator::I64TruncF32U => Instr::I64TruncF32U,
        wasmparser::Operator::I64TruncF64S => Instr::I64TruncF64S,
        wasmparser::Operator::I64TruncF64U => Instr::I64TruncF64U,
        wasmparser::Operator::I32TruncSatF32S => Instr::I32TruncSatF32S,
        wasmparser::Operator::I32TruncSatF32U => Instr::I32TruncSatF32U,
        wasmparser::Operator::I32TruncSatF64S => Instr::I32TruncSatF64S,
        wasmparser::Operator::I32TruncSatF64U => Instr::I32TruncSatF64U,
        wasmparser::Operator::I64TruncSatF32S => Instr::I64TruncSatF32S,
        wasmparser::Operator::I64TruncSatF32U => Instr::I64TruncSatF32U,
        wasmparser::Operator::I64TruncSatF64S => Instr::I64TruncSatF64S,
        wasmparser::Operator::I64TruncSatF64U => Instr::I64TruncSatF64U,
        wasmparser::Operator::F32DemoteF64 => Instr::F32DemoteF64,
        wasmparser::Operator::F64PromoteF32 => Instr::F64PromoteF32,
        wasmparser::Operator::F32ConvertI32S => Instr::F32ConvertI32S,
        wasmparser::Operator::F32ConvertI32U => Instr::F32ConvertI32U,
        wasmparser::Operator::F32ConvertI64S => Instr::F32ConvertI64S,
        wasmparser::Operator::F32ConvertI64U => Instr::F32ConvertI64U,
        wasmparser::Operator::F64ConvertI32S => Instr::F64ConvertI32S,
        wasmparser::Operator::F64ConvertI32U => Instr::F64ConvertI32U,
        wasmparser::Operator::F64ConvertI64S => Instr::F64ConvertI64S,
        wasmparser::Operator::F64ConvertI64U => Instr::F64ConvertI64U,
        wasmparser::Operator::I32ReinterpretF32 => Instr::I32ReinterpretF32,
        wasmparser::Operator::I64ReinterpretF64 => Instr::I64ReinterpretF64,
        wasmparser::Operator::F32ReinterpretI32 => Instr::F32ReinterpretI32,
        wasmparser::Operator::F64ReinterpretI64 => Instr::F64ReinterpretI64,
        /* Reference Instructions */
        wasmparser::Operator::RefNull {..} => Instr::RefNull(RefType::ExternalRef),
        wasmparser::Operator::RefIsNull => Instr::RefIsNull,
        wasmparser::Operator::RefFunc {function_index} => Instr::RefFunc(FuncIdx(function_index)),
        /* Parametric Instructions */
        wasmparser::Operator::Drop =>Instr::Drop,
        wasmparser::Operator::Select =>Instr::Select(None),
        wasmparser::Operator::TypedSelect {ty} => Instr::Select(Some(match_value_type(ty))),
        /* Variable Instructions */
        wasmparser::Operator::LocalGet {local_index} => Instr::LocalGet(LocalIdx(local_index)),
        wasmparser::Operator::LocalSet {local_index} => Instr::LocalSet(LocalIdx(local_index)),
        wasmparser::Operator::LocalTee {local_index} => Instr::LocalTee(LocalIdx(local_index)),
        wasmparser::Operator::GlobalGet {global_index} => Instr::GlobalGet(GlobalIdx(global_index)),
        wasmparser::Operator::GlobalSet {global_index} => Instr::GlobalSet(GlobalIdx(global_index)),
        /* Table Instructions */
        wasmparser::Operator::TableGet {table} => Instr::TableGet(TableIdx(table)),
        wasmparser::Operator::TableSet {table} => Instr::TableSet(TableIdx(table)),
        wasmparser::Operator::TableSize {table} => Instr::TableSize(TableIdx(table)),
        wasmparser::Operator::TableGrow {table} => Instr::TableGrow(TableIdx(table)),
        wasmparser::Operator::TableFill {table} => Instr::TableFill(TableIdx(table)),
        wasmparser::Operator::TableCopy {dst_table, src_table} => Instr::TableCopy(TableIdx(dst_table), TableIdx(src_table)),
        wasmparser::Operator::TableInit {elem_index, table} => Instr::TableInit(TableIdx(elem_index), TableIdx(table)),
        wasmparser::Operator::ElemDrop {elem_index} => Instr::ElemDrop(ElemIdx(elem_index)),
        /* Memory Instructions */
        wasmparser::Operator::I32Load {memarg} => {
            Instr::I32Load(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load {memarg} => {
            Instr::I64Load(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::F32Load {memarg} => {
            Instr::F32Load(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::F64Load {memarg} => {
            Instr::F64Load(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Store {memarg} => {
            Instr::I32Store(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Store {memarg} => {
            Instr::I64Store(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::F32Store {memarg} => {
            Instr::F32Store(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::F64Store {memarg} => {
            Instr::F64Store(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Load8S {memarg} => {
            Instr::I32Load8S(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Load8U {memarg} => {
            Instr::I32Load8U(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load8S {memarg} => {
            Instr::I64Load8S(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load8U {memarg} => {
            Instr::I64Load8U(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Load16S {memarg} => {
            Instr::I32Load16S(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Load16U {memarg} => {
            Instr::I32Load16U(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load16S {memarg} => {
            Instr::I64Load16S(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load16U {memarg} => {
            Instr::I64Load16U(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load32S {memarg} => {
            Instr::I64Load32S(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Load32U {memarg} => {
            Instr::I64Load32U(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Store8 {memarg} => {
            Instr::I32Store8(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Store8 {memarg} => {
            Instr::I64Store8(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I32Store16 {memarg} => {
            Instr::I32Store16(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Store16 {memarg} => {
            Instr::I64Store16(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::I64Store32 {memarg} => {
            Instr::I64Store32(Memarg{
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            })
        },
        wasmparser::Operator::MemorySize{..} => Instr::MemorySize,
        wasmparser::Operator::MemoryGrow{..} => Instr::MemoryGrow,
        wasmparser::Operator::MemoryFill{..} => Instr::MemoryFill,
        wasmparser::Operator::MemoryCopy{..} => Instr::MemoryCopy,
        wasmparser::Operator::MemoryInit{data_index, ..} => Instr::MemoryInit(DataIdx(data_index)),
        wasmparser::Operator::DataDrop{data_index, ..} => Instr::DataDrop(DataIdx(data_index)),
        /*Control Instructions*/
        wasmparser::Operator::Nop => Instr::Nop,
        wasmparser::Operator::Unreachable => Instr::Unreachable,
        wasmparser::Operator::Block{blockty} => {
            // Generate Block instruction without inner instructions
            let ty = match blockty {
                wasmparser::BlockType::Empty => BlockType(None,None),
                wasmparser::BlockType::Type(v) => BlockType(None, Some(match_value_type(v))),
                wasmparser::BlockType::FuncType(idx) =>BlockType(Some(TypeIdx(idx)), None),
            };
            Instr::Block(ty, vec![]) // Empty vec for flat structure
        },
        wasmparser::Operator::Loop{blockty} => {
            // Generate Loop instruction without inner instructions
            let ty = match blockty {
                wasmparser::BlockType::Empty => BlockType(None,None),
                wasmparser::BlockType::Type(v) => BlockType(None, Some(match_value_type(v))),
                wasmparser::BlockType::FuncType(idx) =>BlockType(Some(TypeIdx(idx)), None),
            };
            Instr::Loop(ty, vec![]) // Empty vec for flat structure
        },
        wasmparser::Operator::If{blockty} => {
            // Generate If instruction without inner instructions
            let ty = match blockty {
                wasmparser::BlockType::Empty => BlockType(None,None),
                wasmparser::BlockType::Type(v) => BlockType(None, Some(match_value_type(v))),
                wasmparser::BlockType::FuncType(idx) =>BlockType(Some(TypeIdx(idx)), None),
            };
            Instr::If(ty, vec![], vec![]) // Empty vecs for flat structure
        },
        // Generate markers for Else and End
        wasmparser::Operator::Else => Instr::ElseMarker,
        wasmparser::Operator::End => Instr::EndMarker,
        wasmparser::Operator::Br{relative_depth} => Instr::Br(LabelIdx(relative_depth)),
        wasmparser::Operator::BrIf{relative_depth} => Instr::BrIf(LabelIdx(relative_depth)),
        wasmparser::Operator::BrTable{targets} => {
            Instr::BrTable(
                targets.targets().map(|x| LabelIdx(x.unwrap())).collect::<Vec<LabelIdx>>(),
                LabelIdx(targets.default()))
        },
        wasmparser::Operator::Return => Instr::Return,
        wasmparser::Operator::Call{function_index} => Instr::Call(FuncIdx(function_index)),
        wasmparser::Operator::CallIndirect{table_index, type_index} => Instr::CallIndirect(TableIdx(table_index), TypeIdx(type_index)),
        wasmparser::Operator::I32Extend8S => Instr::I32Extend8S,
        wasmparser::Operator::I32Extend16S => Instr::I32Extend16S,
        wasmparser::Operator::RefEq => todo!(),
        wasmparser::Operator::I64Extend8S => Instr::I64Extend8S,
        wasmparser::Operator::I64Extend16S => Instr::I64Extend16S,
        wasmparser::Operator::I64Extend32S => Instr::I64Extend32S,
        _ => todo!("Unhandled operator in map_operator_to_instr: {:?}", op)

    };
    Ok(instr)
}

pub fn parse_bytecode(mut module: &mut Module, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let parser = Parser::new(0);

    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;

    for payload in parser.parse_all(&buf) {
        match payload? {
            Version { num, encoding:_ ,range:_ } => {
                if num != 0x01{
                    return Err(Box::new(ParserError::VersionError));
                }
            }

            TypeSection(body) => {
                decode_type_section(body, &mut module)?;
            }

            FunctionSection(body) => {
                decode_func_section(body, &mut module)?;
            }

            ImportSection(body) => {
                decode_import_section(body, &mut module)?;
            }
            ExportSection(body) => {
                decode_export_section(body, &mut module)?;

            }

            TableSection(body) => {
                decode_table_section(body, &mut module)?;
            }

            MemorySection(body) => {
                decode_mem_section(body, &mut module)?;
            }

            TagSection(_) => { /* ... */ }

            GlobalSection(body) => {
                decode_global_section(body, &mut module)?;
            }

            StartSection { func, .. } => {
                module.start = Some(Start{
                    func: FuncIdx(func),
                });
            }

            ElementSection(body) => {
                decode_elem_section(body, &mut module)?;
            }

            DataCountSection { .. } => { /* ... */ }
    
            DataSection(body) => {
                decode_data_section(body, &mut module)?;
            }

            CodeSectionStart { .. } => { /* ... */ }
            CodeSectionEntry(body) => {
                decode_code_section(body, &mut module)?;
            }

            ModuleSection { .. } => { /* ... */ }
            InstanceSection(_) => { /* ... */ }
            CoreTypeSection(_) => { /* ... */ }
            ComponentSection { .. } => { /* ... */ }
            ComponentInstanceSection(_) => { /* ... */ }
            ComponentAliasSection(_) => { /* ... */ }
            ComponentTypeSection(_) => { /* ... */ }
            ComponentCanonicalSection(_) => { /* ... */ }
            ComponentStartSection { .. } => { /* ... */ }
            ComponentImportSection(_) => { /* ... */ }
            ComponentExportSection(_) => { /* ... */ }

            CustomSection(_) => { /* ... */ }

            UnknownSection { .. } => { /* ... */ }


            End(_) => {}
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use wasmparser::{Parser, Payload::*, TypeRef, ValType, SectionLimited, ExternalKind, FunctionBody, OperatorsIteratorWithOffsets};
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;

    use crate::structure::{types::*, instructions::*, module::*};
    use crate::error::ParserError;
    use wasmparser::Payload::*;
    use crate::parser;

    #[test]
    fn decode_type_section() {
        let wat = r#"
        (module
            (func (param i32 i32 i64))
            (func (result i32 i32))
            (func (param i32 i32) (result i32))
        )"#;
        
        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                TypeSection(body) => {
                    parser::decode_type_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let len = module.types.len();
        let exptect_param = [3, 0, 2];
        let exptect_result = [0, 2, 1];
        for i in 0..len{
            let params =  &module.types[i].params;
            let results = &module.types[i].results;
            assert_eq!(params.len(), exptect_param[i]);
            assert_eq!(results.len(), exptect_result[i]);
        }
        assert_eq!(len, 3);
    }

    #[test]
    fn decode_func_section(){
        let wat = r#"
        (module
            (import "test" "test" (func (param i32))) 
            (import "test" "test" (func (param i32 i32) (result i32))) 
            (func (param i32 i32 i64))
            (func (result i32 i32)) 
            (func (param i32 i32) (result i32))
        )"#; 
        /*
        Type Idx
        0: (param i32)
        1: (param i32 i32) (result i32)
        2: (param i32 i32 i64)
        3: (result i32 i32)
        */

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                FunctionSection(body) => {
                    parser::decode_func_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let funcs_num = module.funcs.len();
        assert_eq!(funcs_num, 3);

        let exptect_idx = [2, 3, 1];
        for i in 0..funcs_num{
            let idx = &module.funcs[i].type_;
            assert_eq!(idx.0, exptect_idx[i]);
        }


    }

    #[test]
    fn decode_import_section() {
        let wat = r#"
        (module
            (import "module1" "func1" (func (param i32))) 
            (import "module2" "func1" (func (param i32 i32) (result i32)))
            (import "module2" "func2" (func (param i32 i64) (result f32))) 
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                ImportSection(body) => {
                    parser::decode_import_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let imports_num = module.imports.len();
        assert_eq!(imports_num, 3);
        
        let module_names = ["module1", "module2", "module2"];
        let names = ["func1" ,"func1", "func2"];
        for i in 0..imports_num{
            let module_name = &module.imports[i].module.0;
            let name = &module.imports[i].name.0;
            let desc = &module.imports[i].desc;
            assert_eq!(module_name, module_names[i]);
            assert_eq!(name, names[i]);
            assert!(matches!(desc, ImportDesc::Func(TypeIdx(i))));
        }

    }

    #[test]
    fn decode_export_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (func $add (export "add") (param $a i32) (param $b i32) (result i32)
                (i32.add (local.get $a) (local.get $b))
            )
            (func $sub (export "sub") (param $a i32) (param $b i32) (result i32)
                (i32.sub (local.get $a) (local.get $b))
            )
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                ExportSection(body) => {
                    parser::decode_export_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let exports_num = module.exports.len();
        assert_eq!(exports_num, 3);

        let names = ["memory", "add", "sub"];
        for i in 0..exports_num {
            let name = &module.exports[i].name.0;
            let desc = &module.exports[i].desc;
            assert_eq!(name, names[i]);
            let expect = if i == 0 {
                ExportDesc::Mem(MemIdx(i as u32))
            }else{
                ExportDesc::Func(FuncIdx((i-1) as u32))
            };
            assert!(matches!(desc, expect));
        }
    }

    #[test]
    fn decode_mem_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (memory (export "mem") 1)

        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                MemorySection(body) => {
                    parser::decode_mem_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let memory_num = module.mems.len();
        assert_eq!(memory_num, 2);

        let expects_min = [2, 1];
        for i in 0..memory_num {
            let limits = &module.mems[i].type_.0;
            let min = limits.min;
            let max = limits.max;
            assert_eq!(min, expects_min[i]);
            if i ==0 {
                assert_eq!(max, Some(3));
            }else{
                assert_eq!(max, None);

            }
        }
    }

    #[test]
    fn decode_table_section() {
        let wat = r#"
        (module
            (table 2 externref)
            (table 3 funcref)

        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                TableSection(body) => {
                    parser::decode_table_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let table_num = module.tables.len();
        assert_eq!(table_num, 2);

        let mut limits = &module.tables[0].type_.0;
        let mut reftype = &module.tables[0].type_.1;
        let mut min = limits.min;
        let mut max = limits.max;
        assert_eq!(min, 2);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::ExternalRef));

        limits = &module.tables[1].type_.0;
        reftype = &module.tables[1].type_.1;
        min = limits.min;
        max = limits.max;
        assert_eq!(min, 3);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::FuncRef));

    }

    #[test]
    fn decode_global_section() {
        let wat = r#"
        (module
            (global $f32 f32)
            (global $f64 (mut i64)(i64.const 2024))
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                GlobalSection(body) => {
                    parser::decode_global_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let global_num = module.globals.len();
        assert_eq!(global_num, 2);

        let mut type_ = &module.globals[0].type_;
        let mut init = &module.globals[0].init;
        assert!(matches!(type_.0, Mut::Const));
        assert!(matches!(type_.1, ValueType::NumType(NumType::F32)));
        assert_eq!(init.0.len(), 0);


        type_ = &module.globals[1].type_;
        init = &module.globals[1].init;
        assert!(matches!(type_.0, Mut::Var));
        assert!(matches!(type_.1, ValueType::NumType(NumType::I64)));
        assert_eq!(init.0.len(), 1);
        assert!(matches!(init.0[0], Instr::I64Const(2024)));
    }

    #[test]
    fn decode_elem_section() {
        let wat = r#"
        (module
            (table $t0 (export "table1") 2 funcref)
       
            (func $f1 (result i32)
                i32.const 1
            )
            (func $f2 (result i32)
                i32.const 2
            )
            (func $f3 (result i32)
                i32.const 3
            )
            (elem $t0(i32.const 0) $f1 $f2)
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                TableSection(body) =>{
                    parser::decode_table_section(body, &mut module).unwrap();
                }
                ElementSection(body) => {
                    parser::decode_elem_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let elem_num = module.elems.len();
        assert_eq!(elem_num, 1);

        let mut elem = &module.elems[0];
        assert!(matches!(elem.type_,RefType::FuncRef));
        assert!(matches!(elem.mode, ElemMode::Active));
        assert_eq!(elem.table_idx, Some(TableIdx(0)));
    }

    #[test]
    fn decode_data_section() {
        //Test Code: https://github.com/eliben/wasm-wat-samples
        let wat = r#"
        (module
            (memory (export "memory") 1 100)
                (data (i32.const 0x0000)
                    "\67\68\69\70\AA\FF\DF\CB"
                    "\12\A1\32\B3\A5\1F\01\02"
                )
                (data (i32.const 0x020)
                    "\01\03\05\07\09\0B\0D\0F"
                )
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");
    
        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                MemorySection(body) => {
                    parser::decode_mem_section(body, &mut module).unwrap();
                },
                DataSection(body) => {
                    parser::decode_data_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let data_num = module.datas.len();
        assert_eq!(data_num, 2);

        let mut init = &module.datas[0].init;
        let mut mode = &module.datas[0].mode;
        let mut memory = &module.datas[0].memory;
        let mut offset = &module.datas[0].offset;
        assert_eq!(init[0].0, 0x67);
        assert_eq!(init[7].0, 0xCB);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0)]);
        assert_eq!(*offset, Some(expected));

        init = &module.datas[1].init;
        mode = &module.datas[1].mode;
        memory = &module.datas[1].memory;
        offset = &module.datas[1].offset;
        assert_eq!(init[0].0, 0x01);
        assert_eq!(init[7].0, 0x0F);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0x020)]);
        assert_eq!(*offset, Some(expected));

    }

    #[test]
    fn decode_code_section_if_else(){
        //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/if...else
        let wat = r#"
        (module
            (func $ifexpr (result i32)
                i32.const 0
                (if (result i32)
                (then
                    ;; do something
                    (i32.const 1)
                )
                (else
                    ;; do something else
                    (i32.const 2)
                )
                )
            )
        )"#; 

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");
    
        for payload in parser.parse_all(&binary){
            match payload.unwrap() {
                FunctionSection(body) => {
                    parser::decode_func_section(body, &mut module).unwrap();
                },
                CodeSectionEntry(body) => {
                    parser::decode_code_section(body, &mut module).unwrap();
                },
                _ => {},
            }
        };
        let func_num = module.funcs.len();
        assert_eq!(func_num, 1);
        
        let func_body = &module.funcs[0].body;
        println!("{:?}",func_body);
        // Expected value for flat instruction list with markers
        let expected = Expr(vec![
            Instr::I32Const(0),
            Instr::If(BlockType(None, Some(ValueType::NumType(NumType::I32))), vec![], vec![]),
            Instr::I32Const(1),
            Instr::ElseMarker,
            Instr::I32Const(2),
            Instr::EndMarker,
            Instr::EndMarker, // Function body EndMarker
        ]);
        assert_eq!(expected, *func_body);
   }

   #[test]
   fn decode_code_section_loop(){
       //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/loop
       let wat = r#"
       (module
            (func
                (local $i i32)
                (loop $my_loop
                    ;; add one to $i
                    local.get $i
                    i32.const 1
                    i32.add
                    local.set $i
                    ;; if $i is less than 10 branch to loop
                    local.get $i
                    i32.const 10
                    i32.lt_s
                    br_if $my_loop
                )
            )
       )"#; 

       let binary = wat::parse_str(wat).unwrap();
       let parser = wasmparser::Parser::new(0);
       let mut module = Module::new("test");
   
       for payload in parser.parse_all(&binary){
           match payload.unwrap() {
               FunctionSection(body) => {
                   parser::decode_func_section(body, &mut module).unwrap();
               },
               CodeSectionEntry(body) => {
                   parser::decode_code_section(body, &mut module).unwrap();
               },
               _ => {},
           }
       };
       let func_num = module.funcs.len();
       assert_eq!(func_num, 1);
       
       let func_body = &module.funcs[0].body;
       println!("{:?}",func_body);
       // Expected value for flat instruction list with markers
       let expected = Expr(vec![
           Instr::Loop(BlockType(None, None), vec![]),
           Instr::LocalGet(LocalIdx(0)),
           Instr::I32Const(1),
           Instr::I32Add,
           Instr::LocalSet(LocalIdx(0)),
           Instr::LocalGet(LocalIdx(0)),
           Instr::I32Const(10),
           Instr::I32LtS,
           Instr::BrIf(LabelIdx(0)),
           Instr::EndMarker, // End marker for Loop
           Instr::EndMarker, // Function body EndMarker
       ]);
       assert_eq!(expected, *func_body);
  }
}
