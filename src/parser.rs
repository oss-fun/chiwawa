use wasmparser::{Parser, Payload::*, TypeRef, ValType, SectionLimited, ExternalKind, FunctionBody};
use std::fs::File;
use std::io::Read;
use thiserror::Error;

use crate::module::*;
use crate::types::*;
use crate::instructions::*;

#[derive(Debug, Error)]
enum ParserError {
    #[error("Invalid Version")]
    VersionError,
    #[error("Unsupported OP Code in Global Section Init Expr at Offset: {offset}")]
    InitExprUnsupportedOPCodeError{offset: usize},
}

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
            wasmparser::Operator::F32Const {value} => instrs.push(Instr::F32Const(value.bits())),
            wasmparser::Operator::F64Const {value} => instrs.push(Instr::F64Const(value.bits())),
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

        let (type_, init) = match entry.items {
            wasmparser::ElementItems::Functions(funcs) => {
                let mut exprs = Vec::new();
                for func in funcs {
                    let mut inst = Vec::new();
                    inst.push(Instr::RefFunc(FuncIdx(func?)));
                    exprs.push(Expr(inst));
                }
                (RefType::FuncRef, exprs)
            },
            wasmparser::ElementItems::Expressions(type_, items) => {
                let mut exprs = Vec::new();
                for expr in items {
                    let expr = parse_initexpr(expr?)?;
                    exprs.push(expr);
                }

                if type_.is_func_ref() {
                    (RefType::FuncRef, exprs)
                } else {
                    (RefType::ExternalRef, exprs)
                }
            }
        };
        let (mode, table_idx, offset) = match entry.kind {
            wasmparser::ElementKind::Active{table_index, offset_expr} => {
                let expr = parse_initexpr(offset_expr)?;
                (ElemMode::Active, Some(TableIdx(table_index.unwrap_or(0))), Some(expr))
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

pub fn parse_bytecode(mut module: Module, path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            CodeSectionEntry(_body) => {
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