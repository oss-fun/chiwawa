//! Execution tracing for debugging and instrumentation.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use super::value::Val;
use super::vm::*;

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
    Stack,
    Memory,
    Locals,
    Globals,
}

impl TraceResource {
    /// Parses trace resource from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pc" => Some(TraceResource::PC),
            "stack" => Some(TraceResource::Stack),
            "memory" => Some(TraceResource::Memory),
            "locals" => Some(TraceResource::Locals),
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
                TraceResource::Stack,
                TraceResource::Locals,
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
        value_stack: &[Val],
        locals: &[Val],
        global_addrs: &[super::global::GlobalAddr],
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
        let instr_name = Self::get_instruction_name(handler_index);
        parts.push(format!("Instr:{}", instr_name));

        // Stack
        if self.config.resources.contains(&TraceResource::Stack) {
            let stack_str = self.format_stack(value_stack);
            parts.push(format!("Stack:{}", stack_str));
        }

        // Locals
        if self.config.resources.contains(&TraceResource::Locals) {
            let locals_str = self.format_locals(locals);
            parts.push(format!("Locals:{}", locals_str));
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

    fn format_globals(&self, global_addrs: &[super::global::GlobalAddr]) -> String {
        if global_addrs.is_empty() {
            return "[]".to_string();
        }

        let values: Vec<String> = global_addrs
            .iter()
            .map(|g| Self::format_val(&g.get()))
            .collect();
        format!("[{}]", values.join(","))
    }

    fn format_stack(&self, stack: &[Val]) -> String {
        if stack.is_empty() {
            return "[]".to_string();
        }

        let values: Vec<String> = stack.iter().map(|v| Self::format_val(v)).collect();
        format!("[{}]", values.join(","))
    }

    fn format_locals(&self, locals: &[Val]) -> String {
        if locals.is_empty() {
            return "[]".to_string();
        }

        let values: Vec<String> = locals.iter().map(|v| Self::format_val(v)).collect();
        format!("[{}]", values.join(","))
    }

    fn format_val(val: &Val) -> String {
        match val {
            Val::Num(num) => match num {
                super::value::Num::I32(v) => format!("I32({})", v),
                super::value::Num::I64(v) => format!("I64({})", v),
                super::value::Num::F32(v) => format!("F32({})", v),
                super::value::Num::F64(v) => format!("F64({})", v),
            },
            Val::Vec_(vec) => match vec {
                super::value::Vec_::V128(v) => format!("V128({})", v),
            },
            Val::Ref(r) => match r {
                super::value::Ref::RefNull => "RefNull".to_string(),
                super::value::Ref::FuncAddr(_) => "FuncAddr".to_string(),
                super::value::Ref::RefExtern(_) => "RefExtern".to_string(),
            },
        }
    }

    fn get_instruction_name(handler_index: usize) -> &'static str {
        match handler_index {
            // Control Instructions
            HANDLER_IDX_UNREACHABLE => "unreachable",
            HANDLER_IDX_NOP => "nop",
            HANDLER_IDX_BLOCK => "block",
            HANDLER_IDX_LOOP => "loop",
            HANDLER_IDX_IF => "if",
            HANDLER_IDX_ELSE => "else",
            HANDLER_IDX_END => "end",
            HANDLER_IDX_BR => "br",
            HANDLER_IDX_BR_IF => "br_if",
            HANDLER_IDX_BR_TABLE => "br_table",
            HANDLER_IDX_RETURN => "return",
            HANDLER_IDX_CALL => "call",
            HANDLER_IDX_CALL_INDIRECT => "call_indirect",

            // Parametric Instructions
            HANDLER_IDX_DROP => "drop",
            HANDLER_IDX_SELECT => "select",

            // Variable Instructions
            HANDLER_IDX_LOCAL_GET => "local.get",
            HANDLER_IDX_LOCAL_SET => "local.set",
            HANDLER_IDX_LOCAL_TEE => "local.tee",
            HANDLER_IDX_GLOBAL_GET => "global.get",
            HANDLER_IDX_GLOBAL_SET => "global.set",

            // Memory Instructions
            HANDLER_IDX_I32_LOAD => "i32.load",
            HANDLER_IDX_I64_LOAD => "i64.load",
            HANDLER_IDX_F32_LOAD => "f32.load",
            HANDLER_IDX_F64_LOAD => "f64.load",
            HANDLER_IDX_I32_LOAD8_S => "i32.load8_s",
            HANDLER_IDX_I32_LOAD8_U => "i32.load8_u",
            HANDLER_IDX_I32_LOAD16_S => "i32.load16_s",
            HANDLER_IDX_I32_LOAD16_U => "i32.load16_u",
            HANDLER_IDX_I64_LOAD8_S => "i64.load8_s",
            HANDLER_IDX_I64_LOAD8_U => "i64.load8_u",
            HANDLER_IDX_I64_LOAD16_S => "i64.load16_s",
            HANDLER_IDX_I64_LOAD16_U => "i64.load16_u",
            HANDLER_IDX_I64_LOAD32_S => "i64.load32_s",
            HANDLER_IDX_I64_LOAD32_U => "i64.load32_u",
            HANDLER_IDX_I32_STORE => "i32.store",
            HANDLER_IDX_I64_STORE => "i64.store",
            HANDLER_IDX_F32_STORE => "f32.store",
            HANDLER_IDX_F64_STORE => "f64.store",
            HANDLER_IDX_I32_STORE8 => "i32.store8",
            HANDLER_IDX_I32_STORE16 => "i32.store16",
            HANDLER_IDX_I64_STORE8 => "i64.store8",
            HANDLER_IDX_I64_STORE16 => "i64.store16",
            HANDLER_IDX_I64_STORE32 => "i64.store32",
            HANDLER_IDX_MEMORY_SIZE => "memory.size",
            HANDLER_IDX_MEMORY_GROW => "memory.grow",
            HANDLER_IDX_MEMORY_COPY => "memory.copy",
            HANDLER_IDX_MEMORY_INIT => "memory.init",
            HANDLER_IDX_MEMORY_FILL => "memory.fill",

            // Const Instructions
            HANDLER_IDX_I32_CONST => "i32.const",
            HANDLER_IDX_I64_CONST => "i64.const",
            HANDLER_IDX_F32_CONST => "f32.const",
            HANDLER_IDX_F64_CONST => "f64.const",

            // Numeric Instructions - i32
            HANDLER_IDX_I32_EQZ => "i32.eqz",
            HANDLER_IDX_I32_EQ => "i32.eq",
            HANDLER_IDX_I32_NE => "i32.ne",
            HANDLER_IDX_I32_LT_S => "i32.lt_s",
            HANDLER_IDX_I32_LT_U => "i32.lt_u",
            HANDLER_IDX_I32_GT_S => "i32.gt_s",
            HANDLER_IDX_I32_GT_U => "i32.gt_u",
            HANDLER_IDX_I32_LE_S => "i32.le_s",
            HANDLER_IDX_I32_LE_U => "i32.le_u",
            HANDLER_IDX_I32_GE_S => "i32.ge_s",
            HANDLER_IDX_I32_GE_U => "i32.ge_u",
            HANDLER_IDX_I32_CLZ => "i32.clz",
            HANDLER_IDX_I32_CTZ => "i32.ctz",
            HANDLER_IDX_I32_POPCNT => "i32.popcnt",
            HANDLER_IDX_I32_ADD => "i32.add",
            HANDLER_IDX_I32_SUB => "i32.sub",
            HANDLER_IDX_I32_MUL => "i32.mul",
            HANDLER_IDX_I32_DIV_S => "i32.div_s",
            HANDLER_IDX_I32_DIV_U => "i32.div_u",
            HANDLER_IDX_I32_REM_S => "i32.rem_s",
            HANDLER_IDX_I32_REM_U => "i32.rem_u",
            HANDLER_IDX_I32_AND => "i32.and",
            HANDLER_IDX_I32_OR => "i32.or",
            HANDLER_IDX_I32_XOR => "i32.xor",
            HANDLER_IDX_I32_SHL => "i32.shl",
            HANDLER_IDX_I32_SHR_S => "i32.shr_s",
            HANDLER_IDX_I32_SHR_U => "i32.shr_u",
            HANDLER_IDX_I32_ROTL => "i32.rotl",
            HANDLER_IDX_I32_ROTR => "i32.rotr",

            // Numeric Instructions - i64
            HANDLER_IDX_I64_EQZ => "i64.eqz",
            HANDLER_IDX_I64_EQ => "i64.eq",
            HANDLER_IDX_I64_NE => "i64.ne",
            HANDLER_IDX_I64_LT_S => "i64.lt_s",
            HANDLER_IDX_I64_LT_U => "i64.lt_u",
            HANDLER_IDX_I64_GT_S => "i64.gt_s",
            HANDLER_IDX_I64_GT_U => "i64.gt_u",
            HANDLER_IDX_I64_LE_S => "i64.le_s",
            HANDLER_IDX_I64_LE_U => "i64.le_u",
            HANDLER_IDX_I64_GE_S => "i64.ge_s",
            HANDLER_IDX_I64_GE_U => "i64.ge_u",
            HANDLER_IDX_I64_CLZ => "i64.clz",
            HANDLER_IDX_I64_CTZ => "i64.ctz",
            HANDLER_IDX_I64_POPCNT => "i64.popcnt",
            HANDLER_IDX_I64_ADD => "i64.add",
            HANDLER_IDX_I64_SUB => "i64.sub",
            HANDLER_IDX_I64_MUL => "i64.mul",
            HANDLER_IDX_I64_DIV_S => "i64.div_s",
            HANDLER_IDX_I64_DIV_U => "i64.div_u",
            HANDLER_IDX_I64_REM_S => "i64.rem_s",
            HANDLER_IDX_I64_REM_U => "i64.rem_u",
            HANDLER_IDX_I64_AND => "i64.and",
            HANDLER_IDX_I64_OR => "i64.or",
            HANDLER_IDX_I64_XOR => "i64.xor",
            HANDLER_IDX_I64_SHL => "i64.shl",
            HANDLER_IDX_I64_SHR_S => "i64.shr_s",
            HANDLER_IDX_I64_SHR_U => "i64.shr_u",
            HANDLER_IDX_I64_ROTL => "i64.rotl",
            HANDLER_IDX_I64_ROTR => "i64.rotr",

            // Numeric Instructions - f32
            HANDLER_IDX_F32_EQ => "f32.eq",
            HANDLER_IDX_F32_NE => "f32.ne",
            HANDLER_IDX_F32_LT => "f32.lt",
            HANDLER_IDX_F32_GT => "f32.gt",
            HANDLER_IDX_F32_LE => "f32.le",
            HANDLER_IDX_F32_GE => "f32.ge",
            HANDLER_IDX_F32_ABS => "f32.abs",
            HANDLER_IDX_F32_NEG => "f32.neg",
            HANDLER_IDX_F32_CEIL => "f32.ceil",
            HANDLER_IDX_F32_FLOOR => "f32.floor",
            HANDLER_IDX_F32_TRUNC => "f32.trunc",
            HANDLER_IDX_F32_NEAREST => "f32.nearest",
            HANDLER_IDX_F32_SQRT => "f32.sqrt",
            HANDLER_IDX_F32_ADD => "f32.add",
            HANDLER_IDX_F32_SUB => "f32.sub",
            HANDLER_IDX_F32_MUL => "f32.mul",
            HANDLER_IDX_F32_DIV => "f32.div",
            HANDLER_IDX_F32_MIN => "f32.min",
            HANDLER_IDX_F32_MAX => "f32.max",
            HANDLER_IDX_F32_COPYSIGN => "f32.copysign",

            // Numeric Instructions - f64
            HANDLER_IDX_F64_EQ => "f64.eq",
            HANDLER_IDX_F64_NE => "f64.ne",
            HANDLER_IDX_F64_LT => "f64.lt",
            HANDLER_IDX_F64_GT => "f64.gt",
            HANDLER_IDX_F64_LE => "f64.le",
            HANDLER_IDX_F64_GE => "f64.ge",
            HANDLER_IDX_F64_ABS => "f64.abs",
            HANDLER_IDX_F64_NEG => "f64.neg",
            HANDLER_IDX_F64_CEIL => "f64.ceil",
            HANDLER_IDX_F64_FLOOR => "f64.floor",
            HANDLER_IDX_F64_TRUNC => "f64.trunc",
            HANDLER_IDX_F64_NEAREST => "f64.nearest",
            HANDLER_IDX_F64_SQRT => "f64.sqrt",
            HANDLER_IDX_F64_ADD => "f64.add",
            HANDLER_IDX_F64_SUB => "f64.sub",
            HANDLER_IDX_F64_MUL => "f64.mul",
            HANDLER_IDX_F64_DIV => "f64.div",
            HANDLER_IDX_F64_MIN => "f64.min",
            HANDLER_IDX_F64_MAX => "f64.max",
            HANDLER_IDX_F64_COPYSIGN => "f64.copysign",

            // Conversion Instructions
            HANDLER_IDX_I32_WRAP_I64 => "i32.wrap_i64",
            HANDLER_IDX_I32_TRUNC_F32_S => "i32.trunc_f32_s",
            HANDLER_IDX_I32_TRUNC_F32_U => "i32.trunc_f32_u",
            HANDLER_IDX_I32_TRUNC_F64_S => "i32.trunc_f64_s",
            HANDLER_IDX_I32_TRUNC_F64_U => "i32.trunc_f64_u",
            HANDLER_IDX_I64_EXTEND_I32_S => "i64.extend_i32_s",
            HANDLER_IDX_I64_EXTEND_I32_U => "i64.extend_i32_u",
            HANDLER_IDX_I64_TRUNC_F32_S => "i64.trunc_f32_s",
            HANDLER_IDX_I64_TRUNC_F32_U => "i64.trunc_f32_u",
            HANDLER_IDX_I64_TRUNC_F64_S => "i64.trunc_f64_s",
            HANDLER_IDX_I64_TRUNC_F64_U => "i64.trunc_f64_u",
            HANDLER_IDX_F32_CONVERT_I32_S => "f32.convert_i32_s",
            HANDLER_IDX_F32_CONVERT_I32_U => "f32.convert_i32_u",
            HANDLER_IDX_F32_CONVERT_I64_S => "f32.convert_i64_s",
            HANDLER_IDX_F32_CONVERT_I64_U => "f32.convert_i64_u",
            HANDLER_IDX_F32_DEMOTE_F64 => "f32.demote_f64",
            HANDLER_IDX_F64_CONVERT_I32_S => "f64.convert_i32_s",
            HANDLER_IDX_F64_CONVERT_I32_U => "f64.convert_i32_u",
            HANDLER_IDX_F64_CONVERT_I64_S => "f64.convert_i64_s",
            HANDLER_IDX_F64_CONVERT_I64_U => "f64.convert_i64_u",
            HANDLER_IDX_F64_PROMOTE_F32 => "f64.promote_f32",
            HANDLER_IDX_I32_REINTERPRET_F32 => "i32.reinterpret_f32",
            HANDLER_IDX_I64_REINTERPRET_F64 => "i64.reinterpret_f64",
            HANDLER_IDX_F32_REINTERPRET_I32 => "f32.reinterpret_i32",
            HANDLER_IDX_F64_REINTERPRET_I64 => "f64.reinterpret_i64",

            // Sign Extension Instructions
            HANDLER_IDX_I32_EXTEND8_S => "i32.extend8_s",
            HANDLER_IDX_I32_EXTEND16_S => "i32.extend16_s",
            HANDLER_IDX_I64_EXTEND8_S => "i64.extend8_s",
            HANDLER_IDX_I64_EXTEND16_S => "i64.extend16_s",
            HANDLER_IDX_I64_EXTEND32_S => "i64.extend32_s",

            // Saturating Truncation Instructions
            HANDLER_IDX_I32_TRUNC_SAT_F32_S => "i32.trunc_sat_f32_s",
            HANDLER_IDX_I32_TRUNC_SAT_F32_U => "i32.trunc_sat_f32_u",
            HANDLER_IDX_I32_TRUNC_SAT_F64_S => "i32.trunc_sat_f64_s",
            HANDLER_IDX_I32_TRUNC_SAT_F64_U => "i32.trunc_sat_f64_u",
            HANDLER_IDX_I64_TRUNC_SAT_F32_S => "i64.trunc_sat_f32_s",
            HANDLER_IDX_I64_TRUNC_SAT_F32_U => "i64.trunc_sat_f32_u",
            HANDLER_IDX_I64_TRUNC_SAT_F64_S => "i64.trunc_sat_f64_s",
            HANDLER_IDX_I64_TRUNC_SAT_F64_U => "i64.trunc_sat_f64_u",

            // Reference Instructions
            HANDLER_IDX_REF_NULL => "ref.null",
            HANDLER_IDX_REF_IS_NULL => "ref.is_null",

            // Table Instructions
            HANDLER_IDX_TABLE_GET => "table.get",
            HANDLER_IDX_TABLE_SET => "table.set",
            HANDLER_IDX_TABLE_FILL => "table.fill",

            _ => "unknown",
        }
    }
}
