use wasmparser::{Parser, Chunk, Payload::*};
use std::fs::File;
use std::io::Read;


pub fn parse_bytecode(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let parser = Parser::new(0);

    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    
    for payload in parser.parse_all(&buf) {
        match payload? {
            Version { .. } => { /* ... */ }
            TypeSection(_) => { /* ... */ }
            ImportSection(_) => { /* ... */ }
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
            CodeSectionEntry(body) => {
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

            UnknownSection { id, .. } => { /* ... */ }


            End(_) => {}
        }
    }

    Ok(())
}