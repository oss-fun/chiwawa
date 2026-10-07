//! Execution tracing for debugging and instrumentation.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::execution::global::GlobalAddr;
use crate::execution::handlers::*;
use crate::execution::regs::RegFile;
use crate::execution::value::Val;
use crate::instrument::instruction_name;

/// Event types that can trigger tracing.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceEvent {
    All,
    Store,
    Load,
    Call,
    Branch,
}

impl TraceEvent {
    /// Parses trace event from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "all" => Some(TraceEvent::All),
            "store" => Some(TraceEvent::Store),
            "load" => Some(TraceEvent::Load),
            "call" => Some(TraceEvent::Call),
            "branch" => Some(TraceEvent::Branch),
            _ => None,
        }
    }
}

/// Resources to include in trace output.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceResource {
    PC,
    Regs,
    Memory,
    Globals,
}

impl TraceResource {
    /// Parses trace resource from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pc" => Some(TraceResource::PC),
            "regs" => Some(TraceResource::Regs),
            "memory" => Some(TraceResource::Memory),
            "globals" => Some(TraceResource::Globals),
            _ => None,
        }
    }
}

/// Configuration for execution tracing.
#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub events: Vec<TraceEvent>,
    pub resources: Vec<TraceResource>,
    pub output_path: Option<String>,
}

impl TraceConfig {
    /// Creates trace configuration from optional string arguments.
    pub fn new(
        events: Option<Vec<String>>,
        resources: Option<Vec<String>>,
        output_path: Option<String>,
    ) -> Self {
        let events = if let Some(event_strs) = events {
            event_strs
                .iter()
                .filter_map(|s| TraceEvent::from_str(s))
                .collect()
        } else {
            vec![TraceEvent::All]
        };

        let resources = if let Some(resource_strs) = resources {
            resource_strs
                .iter()
                .filter_map(|s| TraceResource::from_str(s))
                .collect()
        } else {
            vec![
                TraceResource::PC,
                TraceResource::Regs,
                TraceResource::Globals,
            ]
        };

        Self {
            events,
            resources,
            output_path,
        }
    }

    /// Returns true if the given instruction should be traced.
    pub fn should_trace_event(&self, handler_index: usize) -> bool {
        if self.events.contains(&TraceEvent::All) {
            return true;
        }

        // Check if the instruction matches any trace event
        for event in &self.events {
            match event {
                TraceEvent::Store => {
                    if (HANDLER_IDX_I32_STORE..=HANDLER_IDX_I64_STORE32).contains(&handler_index) {
                        return true;
                    }
                }
                TraceEvent::Load => {
                    if (HANDLER_IDX_I32_LOAD..=HANDLER_IDX_I64_LOAD32_U).contains(&handler_index) {
                        return true;
                    }
                }
                TraceEvent::Call => {
                    if handler_index == HANDLER_IDX_CALL
                        || handler_index == HANDLER_IDX_CALL_INDIRECT
                    {
                        return true;
                    }
                }
                TraceEvent::Branch => {
                    if handler_index == HANDLER_IDX_BR
                        || handler_index == HANDLER_IDX_BR_IF
                        || handler_index == HANDLER_IDX_BR_TABLE
                    {
                        return true;
                    }
                }
                TraceEvent::All => return true,
            }
        }

        false
    }
}

/// Writes execution traces to file or stderr.
pub struct Tracer {
    config: TraceConfig,
    output: Box<dyn Write>,
}

impl Tracer {
    /// Creates a new tracer with the given configuration.
    pub fn new(config: TraceConfig) -> io::Result<Self> {
        let output: Box<dyn Write> = if let Some(ref path) = config.output_path {
            Box::new(File::create(Path::new(path))?)
        } else {
            Box::new(io::stderr())
        };

        Ok(Self { config, output })
    }

    /// Records a single instruction execution to the trace output.
    pub fn trace_instruction(
        &mut self,
        ip: usize,
        handler_index: usize,
        reg_file: &RegFile,
        global_addrs: &[GlobalAddr],
    ) {
        if !self.config.should_trace_event(handler_index) {
            return;
        }

        let mut parts = Vec::new();

        // PC (instruction pointer)
        if self.config.resources.contains(&TraceResource::PC) {
            parts.push(format!("PC:{:04}", ip));
        }

        // Instruction name
        let instr_name = instruction_name(handler_index);
        parts.push(format!("Instr:{}", instr_name));

        // Registers
        if self.config.resources.contains(&TraceResource::Regs) {
            let regs_str = self.format_registers(reg_file);
            parts.push(format!("Regs:{}", regs_str));
        }

        // Globals
        if self.config.resources.contains(&TraceResource::Globals) {
            let globals_str = self.format_globals(global_addrs);
            parts.push(format!("Globals:{}", globals_str));
        }

        // Write trace line
        let trace_line = format!("[{}]\n", parts.join(" | "));
        let _ = self.output.write_all(trace_line.as_bytes());
        let _ = self.output.flush();
    }

    fn format_globals(&self, global_addrs: &[GlobalAddr]) -> String {
        if global_addrs.is_empty() {
            return "[]".to_string();
        }

        let values: Vec<String> = global_addrs
            .iter()
            .map(|g| Self::format_val(&g.get()))
            .collect();
        format!("[{}]", values.join(","))
    }

    fn format_registers(&self, reg_file: &RegFile) -> String {
        let mut parts = Vec::new();

        // Format I32 registers
        if !reg_file.i32_regs.is_empty() {
            let i32_vals: Vec<String> = reg_file
                .i32_regs
                .iter()
                .enumerate()
                .map(|(i, v)| format!("r{}:{}", i, v))
                .collect();
            parts.push(format!("I32[{}]", i32_vals.join(",")));
        }

        // Format I64 registers
        if !reg_file.i64_regs.is_empty() {
            let i64_vals: Vec<String> = reg_file
                .i64_regs
                .iter()
                .enumerate()
                .map(|(i, v)| format!("r{}:{}", i, v))
                .collect();
            parts.push(format!("I64[{}]", i64_vals.join(",")));
        }

        // Format F32 registers
        if !reg_file.f32_regs.is_empty() {
            let f32_vals: Vec<String> = reg_file
                .f32_regs
                .iter()
                .enumerate()
                .map(|(i, v)| format!("r{}:{}", i, v))
                .collect();
            parts.push(format!("F32[{}]", f32_vals.join(",")));
        }

        // Format F64 registers
        if !reg_file.f64_regs.is_empty() {
            let f64_vals: Vec<String> = reg_file
                .f64_regs
                .iter()
                .enumerate()
                .map(|(i, v)| format!("r{}:{}", i, v))
                .collect();
            parts.push(format!("F64[{}]", f64_vals.join(",")));
        }

        if parts.is_empty() {
            "[]".to_string()
        } else {
            format!("{{{}}}", parts.join(", "))
        }
    }

    fn format_val(val: &Val) -> String {
        match val {
            Val::Num(num) => match num {
                crate::execution::value::Num::I32(v) => format!("I32({})", v),
                crate::execution::value::Num::I64(v) => format!("I64({})", v),
                crate::execution::value::Num::F32(v) => format!("F32({})", v),
                crate::execution::value::Num::F64(v) => format!("F64({})", v),
            },
            Val::Vec_(vec) => match vec {
                crate::execution::value::Vec_::V128(v) => format!("V128({})", v),
            },
            Val::Ref(r) => match r {
                crate::execution::value::Ref::RefNull => "RefNull".to_string(),
                crate::execution::value::Ref::FuncAddr(_) => "FuncAddr".to_string(),
                crate::execution::value::Ref::RefExtern(_) => "RefExtern".to_string(),
            },
        }
    }
}
