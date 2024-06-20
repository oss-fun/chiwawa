use wasmparser::{Parser, Payload::*, TypeRef, ValType, SectionLimited, ExternalKind};
use std::fs::File;
use std::io::Read;
use thiserror::Error;

use crate::module::*;
use crate::types::*;

#[derive(Debug, Error)]
enum ParserError {
    #[error("Invalid Version")]
    VersionError,
}

fn types_to_vec(types: &[ValType], vec: &mut Vec<ValueType>){
    for t in types.iter(){
        let type_ = match t {
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
            }
        };
        vec.push(type_);
    }
}

fn decode_type_section(body: SectionLimited<'_, wasmparser::RecGroup>, functypes: &mut Vec<FuncType>) -> Result<(), Box<dyn std::error::Error>>{
    for functype in body.into_iter_err_on_gc_types() {
        let functype = functype?;

        let mut params = Vec::new();
        let mut results = Vec::new();
        types_to_vec(functype.params(), &mut params);
        types_to_vec(functype.results(), &mut results);

        let r = functype.results();
        functypes.push(FuncType{
                params,
                results
        });
    };
    Ok(())
}

fn decode_func_section(body: SectionLimited<'_, u32>, types: &mut Vec<TypeIdx>) -> Result<(), Box<dyn std::error::Error>>{
    for func in body{
        let index = func?;
        let typeidx = TypeIdx(index);
        types.push(typeidx);
    }

    Ok(())
}

fn decode_import_section(body: SectionLimited<'_, wasmparser::Import<'_>>, imports: &mut Vec<Import>) -> Result<(), Box<dyn std::error::Error>>{
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
                let value_type = match global.content_type {
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
                    }
                };
                ImportDesc::Global(GlobalType(mut_,value_type))
            },
            TypeRef::Tag(_) => todo!()
        };
        imports.push(
            Import{
                module: Name(import.module.to_string()),
                name: Name(import.name.to_string()),
                desc,
            }
        );
    }
    Ok(())
}

fn decode_export_section(body: SectionLimited<'_, wasmparser::Export<'_>>, exports: &mut Vec<Export>) -> Result<(), Box<dyn std::error::Error>>{
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
        exports.push(
            Export{
                name: Name(export.name.to_string()),
                desc,
            }
        );
    };
    Ok(())
}

fn decode_mem_section(body: SectionLimited<'_, wasmparser::MemoryType>, mems: &mut Vec<Mem>) -> Result<(), Box<dyn std::error::Error>>{

    for memory in body {
        let memory = memory?;
        let max = match memory.maximum {
            Some(m) => Some(TryFrom::try_from(m).unwrap()),
            None => None
        };
        let limits = Limits{min: TryFrom::try_from(memory.initial).unwrap(), max};
        mems.push(Mem{
            type_: MemType(limits)
        });
    }
    Ok(())
}

fn decode_table_section(body: SectionLimited<'_, wasmparser::Table<'_>>, tables: &mut Vec<Table>) -> Result<(), Box<dyn std::error::Error>>{
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
        tables.push(Table{
            type_: TableType(limits,reftype)
        });
    }
    Ok(())
}

pub fn parse_bytecode(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let parser = Parser::new(0);

    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;

    let mut types = Vec::new();
    let mut func_typeidx = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut tables = Vec::new();
    let mut mems = Vec::new();

    for payload in parser.parse_all(&buf) {
        match payload? {
            Version { num, encoding ,range } => {
                if num != 0x01{
                    return Err(Box::new(ParserError::VersionError));
                }
            }

            TypeSection(body) => {
                let _= decode_type_section(body, &mut types);
            }

            FunctionSection(body) => {
                let _ = decode_func_section(body,&mut func_typeidx);
            }

            ImportSection(body) => {
                let _ = decode_import_section(body, &mut imports);
            }
            ExportSection(body) => {
                let _ = decode_export_section(body, &mut exports);

            }

            TableSection(body) => {
                let _ = decode_table_section(body, &mut tables);
            }

            MemorySection(body) => {
                let _ = decode_mem_section(body, &mut mems);
            }

            TagSection(_) => { /* ... */ }
            GlobalSection(_) => { /* ... */ }
            StartSection { .. } => { /* ... */ }
            ElementSection(_) => { /* ... */ }
            DataCountSection { .. } => { /* ... */ }
            DataSection(_) => { /* ... */ }

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