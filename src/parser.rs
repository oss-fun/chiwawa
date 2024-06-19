use wasmparser::{Parser, Payload::*, TypeRef, ValType,SectionLimited};
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

pub fn parse_bytecode(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let parser = Parser::new(0);

    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;

    let mut imports = Vec::new();

    for payload in parser.parse_all(&buf) {
        match payload? {
            Version { num, encoding ,range } => {
                if num != 0x01{
                    return Err(Box::new(ParserError::VersionError));
                }
            }

            TypeSection(_) => {
            }

            ImportSection(body) => {
                let _ = decode_import_section(body,&mut imports);
            }

            FunctionSection(_) => { /* ... */ }
            TableSection(_) => { /* ... */ }
            MemorySection(_) => { /* ... */ }
            TagSection(_) => { /* ... */ }
            GlobalSection(_) => { /* ... */ }
            ExportSection(_) => { /* ... */ }
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