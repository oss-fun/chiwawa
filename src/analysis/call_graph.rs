//! Static call graph built from processed Wasm bytecode.
//!
//! Nodes are function indices (same space as FuncIdx):
//!   0..module.num_imported_funcs      → imported/WASI functions (no outgoing edges)
//!   num_imported_funcs..num_funcs     → locally-defined functions
//!
//! `call_indirect` edges use type-based conservative approximation:
//! for each `CallIndirectReg { type_idx }`, all non-WASI functions with that type
//! become candidate callees.

use crate::execution::ir::ProcessedInstr;
use crate::structure::module::{ImportDesc, Module};
use crate::structure::types::{FuncIdx, TypeIdx};
use rustc_hash::{FxHashMap, FxHashSet};

/// Static call graph over a parsed Wasm module.
pub struct CallGraph {
    /// `callees[raw_func_idx]` = deduplicated list of potential callees.
    /// Imported functions always have an empty list (no body to scan).
    callees: Vec<Vec<FuncIdx>>,
    num_funcs: usize,
}

impl CallGraph {
    /// Build a call graph from a fully-parsed module.
    pub fn build(module: &Module) -> Self {
        let num_imported = module.num_imported_funcs;
        let num_local = module.funcs.len();
        let num_funcs = num_imported + num_local;

        // Step 1: TypeIdx → candidate FuncIdx list for call_indirect approximation.
        let mut type_to_funcs: FxHashMap<TypeIdx, Vec<FuncIdx>> = FxHashMap::default();

        let mut import_func_idx: u32 = 0;
        for import in &module.imports {
            match &import.desc {
                ImportDesc::Func(type_idx) => {
                    type_to_funcs
                        .entry(*type_idx)
                        .or_default()
                        .push(FuncIdx(import_func_idx));
                    import_func_idx += 1;
                }
                ImportDesc::WasiFunc(_) => {
                    import_func_idx += 1;
                }
                _ => {}
            }
        }
        for (local_idx, func) in module.funcs.iter().enumerate() {
            let func_idx = FuncIdx((num_imported + local_idx) as u32);
            type_to_funcs.entry(func.type_).or_default().push(func_idx);
        }

        // Step 2: scan each local function body for call instructions.
        let mut callees = vec![Vec::new(); num_funcs];

        for (local_idx, func) in module.funcs.iter().enumerate() {
            let caller_raw = num_imported + local_idx;
            let mut callee_set: FxHashSet<u32> = FxHashSet::default();

            for instr in func.body.iter() {
                match instr {
                    ProcessedInstr::CallReg { func_idx, .. } => {
                        callee_set.insert(func_idx.0);
                    }
                    ProcessedInstr::CallIndirectReg { type_idx, .. } => {
                        if let Some(candidates) = type_to_funcs.get(type_idx) {
                            for c in candidates {
                                callee_set.insert(c.0);
                            }
                        }
                    }
                    ProcessedInstr::CallWasiReg { .. } => {}
                    _ => {}
                }
            }

            callees[caller_raw] = callee_set.into_iter().map(FuncIdx).collect();
        }

        CallGraph { callees, num_funcs }
    }

    /// Total number of functions in the module (imported + local).
    pub fn num_funcs(&self) -> usize {
        self.num_funcs
    }
}
