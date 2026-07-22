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
use crate::structure::module::{ExportDesc, ImportDesc, Module};
use crate::structure::types::{FuncIdx, TypeIdx};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs::File;
use std::io::{BufWriter, Write};

/// Static call graph over a parsed Wasm module.
pub struct CallGraph {
    /// `callees[raw_func_idx]` = deduplicated list of potential callees.
    /// Imported functions always have an empty list (no body to scan).
    callees: Vec<Vec<FuncIdx>>,
    num_funcs: usize,
}

impl CallGraph {
    /// Build a call graph from a fully-parsed module.
    ///
    /// Internally uses [`CallGraphBuilder`]. For a single-pass alternative that
    /// avoids re-scanning function bodies, integrate `CallGraphBuilder` directly
    /// into the parser.
    pub fn build(module: &Module) -> Self {
        let num_imported = module.num_imported_funcs;
        let mut builder = CallGraphBuilder::new();

        // Register all functions in FuncIdx order.
        let mut import_func_idx: u32 = 0;
        for import in &module.imports {
            match &import.desc {
                ImportDesc::Func(type_idx) => {
                    builder.register_import_func(FuncIdx(import_func_idx), *type_idx);
                    import_func_idx += 1;
                }
                ImportDesc::WasiFunc(_) => {
                    builder.register_wasi_func();
                    import_func_idx += 1;
                }
                _ => {}
            }
        }
        for (local_idx, func) in module.funcs.iter().enumerate() {
            builder.register_local_func(FuncIdx((num_imported + local_idx) as u32), func.type_);
        }

        // Scan local function bodies for call instructions.
        for (local_idx, func) in module.funcs.iter().enumerate() {
            let caller = FuncIdx((num_imported + local_idx) as u32);
            for instr in func.body.iter() {
                match instr {
                    ProcessedInstr::CallReg { func_idx, .. } => {
                        builder.record_call(caller, *func_idx);
                    }
                    ProcessedInstr::CallIndirectReg { type_idx, .. } => {
                        builder.record_call_indirect(caller, *type_idx);
                    }
                    ProcessedInstr::CallWasiReg { .. } => {}
                    _ => {}
                }
            }
        }

        builder.finish()
    }

    /// Total number of functions in the module (imported + local).
    pub fn num_funcs(&self) -> usize {
        self.num_funcs
    }

    /// Write the call graph in DOT format to the given file path.
    pub fn report(&self, module: &Module, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut w = BufWriter::new(file);

        // Build function name table indexed by raw FuncIdx value.
        let mut names: Vec<String> = Vec::with_capacity(self.num_funcs);

        // Imported function names: "module::name"
        for import in &module.imports {
            match &import.desc {
                ImportDesc::Func(_) | ImportDesc::WasiFunc(_) => {
                    names.push(format!("{}::{}", import.module.0, import.name.0));
                }
                _ => {}
            }
        }

        // Local function names: export name if available, else "func_N"
        let mut export_names: FxHashMap<u32, &str> = FxHashMap::default();
        for export in &module.exports {
            if let ExportDesc::Func(fidx) = &export.desc {
                export_names.entry(fidx.0).or_insert(export.name.0.as_str());
            }
        }
        for (local_idx, _) in module.funcs.iter().enumerate() {
            let raw_idx = (module.num_imported_funcs + local_idx) as u32;
            let name = export_names
                .get(&raw_idx)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("func_{}", raw_idx));
            names.push(name);
        }

        writeln!(w, "digraph call_graph {{")?;

        // Declare import/WASI nodes with a distinct style.
        for i in 0..module.num_imported_funcs {
            writeln!(
                w,
                "  {:?} [shape=box, style=filled, fillcolor=lightgray];",
                names[i]
            )?;
        }

        // Output edges.
        for caller in 0..self.num_funcs {
            for callee in &self.callees[caller] {
                writeln!(
                    w,
                    "  {:?} -> {:?};",
                    names[caller], names[callee.0 as usize]
                )?;
            }
        }

        writeln!(w, "}}")?;
        Ok(())
    }
}

/// Incremental call graph builder for use during Wasm bytecode parsing.
///
/// Functions must be registered in FuncIdx order (imports first, then locals).
/// Once all functions are registered and call instructions recorded, call
/// [`finish`](CallGraphBuilder::finish) to obtain a [`CallGraph`].
///
/// # Parser integration
///
/// ```ignore
/// let mut builder = CallGraphBuilder::new();
///
/// // ImportSection: for each function import
/// builder.register_import_func(func_idx, type_idx); // non-WASI
/// builder.register_wasi_func();                      // WASI
///
/// // FunctionSection: for each local function
/// builder.register_local_func(func_idx, type_idx);
///
/// // CodeSection: for each call instruction
/// builder.record_call(caller, callee);
/// builder.record_call_indirect(caller, type_idx);
///
/// let call_graph = builder.finish();
/// ```
pub struct CallGraphBuilder {
    type_to_funcs: FxHashMap<TypeIdx, Vec<FuncIdx>>,
    callee_sets: Vec<FxHashSet<u32>>,
    num_funcs: usize,
}

impl Default for CallGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CallGraphBuilder {
    pub fn new() -> Self {
        CallGraphBuilder {
            type_to_funcs: FxHashMap::default(),
            callee_sets: Vec::new(),
            num_funcs: 0,
        }
    }

    /// Register a non-WASI imported function.
    pub fn register_import_func(&mut self, func_idx: FuncIdx, type_idx: TypeIdx) {
        self.callee_sets.push(FxHashSet::default());
        self.num_funcs += 1;
        self.type_to_funcs
            .entry(type_idx)
            .or_default()
            .push(func_idx);
    }

    /// Register a WASI imported function.
    /// WASI functions are excluded from `call_indirect` candidates because they
    /// have no Wasm body to discard.
    pub fn register_wasi_func(&mut self) {
        self.callee_sets.push(FxHashSet::default());
        self.num_funcs += 1;
    }

    /// Register a locally-defined function.
    pub fn register_local_func(&mut self, func_idx: FuncIdx, type_idx: TypeIdx) {
        self.callee_sets.push(FxHashSet::default());
        self.num_funcs += 1;
        self.type_to_funcs
            .entry(type_idx)
            .or_default()
            .push(func_idx);
    }

    /// Record a direct call from `caller` to `callee`.
    pub fn record_call(&mut self, caller: FuncIdx, callee: FuncIdx) {
        if let Some(set) = self.callee_sets.get_mut(caller.0 as usize) {
            set.insert(callee.0);
        }
    }

    /// Record a `call_indirect` from `caller` with the given type signature.
    /// Adds edges to all registered functions whose type matches `type_idx`.
    pub fn record_call_indirect(&mut self, caller: FuncIdx, type_idx: TypeIdx) {
        if let Some(candidates) = self.type_to_funcs.get(&type_idx) {
            let candidates: Vec<u32> = candidates.iter().map(|f| f.0).collect();
            if let Some(set) = self.callee_sets.get_mut(caller.0 as usize) {
                set.extend(candidates);
            }
        }
    }

    /// Consume the builder and produce a [`CallGraph`].
    pub fn finish(self) -> CallGraph {
        let callees = self
            .callee_sets
            .into_iter()
            .map(|set| set.into_iter().map(FuncIdx).collect())
            .collect();
        CallGraph {
            callees,
            num_funcs: self.num_funcs,
        }
    }
}
