//! The canonical ABI: values crossing between component types and core registers or memory.
//!
//! Pointers are 32-bit, since chiwawa has no 64-bit memories, and strings are UTF-8 only.

use super::func::{FuncAddr, FuncInst};
use super::mem::MemAddr;
use super::runtime::{Runtime, RuntimeConfig};
use super::value::{Num, Val};
use crate::error::RuntimeError;
use crate::shared::Shared;
use crate::structure::component::{DefinedType, FuncType, StringEncoding, ValType};
use crate::structure::types::{FuncType as CoreFuncType, NumType, ValueType};
use std::cell::Cell;
use std::rc::Rc;

type Result<T> = std::result::Result<T, RuntimeError>;

pub const MAX_FLAT_PARAMS: usize = 16;
pub const MAX_FLAT_RESULTS: usize = 1;
const MAX_STRING_BYTE_LENGTH: u64 = (1 << 28) - 1;
const MAX_LIST_BYTE_LENGTH: u64 = (1 << 28) - 1;

const I32: ValueType = ValueType::NumType(NumType::I32);
const I64: ValueType = ValueType::NumType(NumType::I64);
const F32: ValueType = ValueType::NumType(NumType::F32);
const F64: ValueType = ValueType::NumType(NumType::F64);

fn trap(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ComponentError(message.into())
}

fn unsupported(feature: &'static str) -> RuntimeError {
    RuntimeError::UnsupportedComponentFeature(feature)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentVal {
    Bool(bool),
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    S64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
    List(Vec<ComponentVal>),
    Record(Vec<(String, ComponentVal)>),
    Tuple(Vec<ComponentVal>),
    Variant(String, Option<Box<ComponentVal>>),
    Enum(String),
    Option(Option<Box<ComponentVal>>),
    Result(std::result::Result<Option<Box<ComponentVal>>, Option<Box<ComponentVal>>>),
    Flags(Vec<String>),
}

pub fn call_core(func: &FuncAddr, params: Vec<Val>) -> Result<Vec<Val>> {
    match func.read_lock() {
        FuncInst::RuntimeFunc { module, .. } => {
            let inst = module
                .upgrade()
                .ok_or_else(|| trap("the function's instance is gone"))?;
            Runtime::new(inst, func, params, RuntimeConfig::default())?.run()
        }
        FuncInst::HostFunc { host_code, .. } => Ok(host_code(params)?.into_iter().collect()),
        FuncInst::WasiFunc { .. } => Err(RuntimeError::UnimplementedHostFunction),
    }
}

/// Clears `may_leave` and restores it when dropped, so a trap cannot leave it cleared.
pub struct LeaveGuard<'a>(&'a Cell<bool>);

impl<'a> LeaveGuard<'a> {
    pub fn new(may_leave: &'a Cell<bool>) -> Self {
        may_leave.set(false);
        LeaveGuard(may_leave)
    }
}

impl Drop for LeaveGuard<'_> {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

/// The resolved options of one `canon lift` or `canon lower`, and the flag of the instance they belong to.
#[derive(Clone)]
pub struct CanonContext {
    pub memory: Option<MemAddr>,
    pub realloc: Option<FuncAddr>,
    pub encoding: StringEncoding,
    /// False while the instance runs `realloc` or `post-return`, when it may not call out.
    pub may_leave: Rc<Cell<bool>>,
}

impl CanonContext {
    fn memory(&self) -> Result<&MemAddr> {
        self.memory
            .as_ref()
            .ok_or_else(|| trap("the memory option is required"))
    }

    /// Traps unless `len` bytes at `ptr` lie inside the memory.
    fn check_range(&self, ptr: u32, len: u64) -> Result<()> {
        if ptr as u64 + len > self.memory()?.data_len() as u64 {
            return Err(trap("out of bounds of memory"));
        }
        Ok(())
    }

    fn read(&self, ptr: u32, len: u32) -> Result<Vec<u8>> {
        self.check_range(ptr, len as u64)?;
        let data = &self.memory()?.get_memory_direct_access().data;
        Ok(data[ptr as usize..(ptr + len) as usize].to_vec())
    }

    fn write(&self, ptr: u32, bytes: &[u8]) -> Result<()> {
        self.check_range(ptr, bytes.len() as u64)?;
        self.memory()?.store_bytes(ptr as i32, bytes);
        Ok(())
    }

    /// Calls the instance's `realloc` with `may_leave` cleared, and checks the pointer it returns.
    fn reallocate(&self, old: u32, old_len: u32, align: u32, new_len: u32) -> Result<u32> {
        let realloc = self
            .realloc
            .as_ref()
            .ok_or_else(|| trap("the realloc option is required"))?;
        let args = [old, old_len, align, new_len]
            .iter()
            .map(|v| Val::Num(Num::I32(*v as i32)))
            .collect();
        let results = {
            let _guard = LeaveGuard::new(&self.may_leave);
            call_core(realloc, args)?
        };
        let ptr = results
            .first()
            .ok_or_else(|| trap("realloc returned nothing"))?
            .to_i32()? as u32;
        if !ptr.is_multiple_of(align) {
            return Err(trap("realloc return: result not aligned"));
        }
        if ptr as u64 + new_len as u64 > self.memory()?.data_len() as u64 {
            return Err(trap("realloc return: beyond end of memory"));
        }
        Ok(ptr)
    }

    fn allocate(&self, align: u32, len: u32) -> Result<u32> {
        self.reallocate(0, 0, align, len)
    }
}

pub fn check_encoding(cx: &CanonContext, ty: &FuncType) -> Result<()> {
    let needs_strings = ty
        .params
        .iter()
        .map(|(_, t)| t)
        .chain(ty.result.iter())
        .any(contains_string);
    if needs_strings && cx.encoding != StringEncoding::Utf8 {
        return Err(unsupported("string transcoding"));
    }
    Ok(())
}

fn contains_string(ty: &ValType) -> bool {
    match ty {
        ValType::String => true,
        ValType::Defined(d) => match &**d {
            DefinedType::List(t) | DefinedType::Option(t) => contains_string(t),
            DefinedType::Record(fields) => fields.iter().any(|(_, t)| contains_string(t)),
            DefinedType::Tuple(ts) => ts.iter().any(contains_string),
            DefinedType::Variant(cases) => cases
                .iter()
                .any(|(_, t)| t.as_ref().is_some_and(contains_string)),
            DefinedType::Result { ok, err } => {
                ok.as_ref().is_some_and(contains_string)
                    || err.as_ref().is_some_and(contains_string)
            }
            DefinedType::Flags(_)
            | DefinedType::Enum(_)
            | DefinedType::Own(_)
            | DefinedType::Borrow(_) => false,
        },
        _ => false,
    }
}

/// Despecializes an enum, option or result into a variant and returns its cases in order: the label and the payload type.
fn despecialized_cases(d: &DefinedType) -> Option<Vec<(&str, Option<&ValType>)>> {
    Some(match d {
        DefinedType::Variant(cases) => cases
            .iter()
            .map(|(label, t)| (label.as_str(), t.as_ref()))
            .collect(),
        DefinedType::Enum(labels) => labels.iter().map(|l| (l.as_str(), None)).collect(),
        DefinedType::Option(t) => vec![("none", None), ("some", Some(t))],
        DefinedType::Result { ok, err } => vec![("ok", ok.as_ref()), ("error", err.as_ref())],
        _ => return None,
    })
}

/// Despecializes a tuple into a record and returns the field types in order.
fn record_fields(d: &DefinedType) -> Option<Vec<&ValType>> {
    Some(match d {
        DefinedType::Record(fields) => fields.iter().map(|(_, t)| t).collect(),
        DefinedType::Tuple(ts) => ts.iter().collect(),
        _ => return None,
    })
}

/// Builds the value of case `label` of `d` with `payload`.
fn case_value(d: &DefinedType, label: &str, payload: Option<Box<ComponentVal>>) -> ComponentVal {
    match d {
        DefinedType::Enum(_) => ComponentVal::Enum(label.to_string()),
        DefinedType::Option(_) => {
            ComponentVal::Option(if label == "some" { payload } else { None })
        }
        DefinedType::Result { .. } => ComponentVal::Result(if label == "ok" {
            Ok(payload)
        } else {
            Err(payload)
        }),
        _ => ComponentVal::Variant(label.to_string(), payload),
    }
}

/// The case index and payload of a value of the variant-like type `d`.
fn case_of<'v>(
    d: &DefinedType,
    val: &'v ComponentVal,
) -> Result<(usize, Option<&'v ComponentVal>)> {
    let position = |labels: &[String], label: &str| {
        labels
            .iter()
            .position(|l| l == label)
            .ok_or_else(|| trap(format!("unknown case {label}")))
    };
    Ok(match (d, val) {
        (DefinedType::Variant(cases), ComponentVal::Variant(label, payload)) => {
            let index = cases
                .iter()
                .position(|(l, _)| l == label)
                .ok_or_else(|| trap(format!("unknown case {label}")))?;
            (index, payload.as_deref())
        }
        (DefinedType::Enum(labels), ComponentVal::Enum(label)) => (position(labels, label)?, None),
        (DefinedType::Option(_), ComponentVal::Option(None)) => (0, None),
        (DefinedType::Option(_), ComponentVal::Option(Some(v))) => (1, Some(&**v)),
        (DefinedType::Result { .. }, ComponentVal::Result(Ok(v))) => (0, v.as_deref()),
        (DefinedType::Result { .. }, ComponentVal::Result(Err(v))) => (1, v.as_deref()),
        _ => return Err(RuntimeError::TypeMismatch),
    })
}

// ---- Layout ----

fn discriminant_size(n: usize) -> u32 {
    if n <= 256 {
        1
    } else if n <= 65536 {
        2
    } else {
        4
    }
}

fn flags_size(n: usize) -> u32 {
    if n <= 8 {
        1
    } else if n <= 16 {
        2
    } else {
        4
    }
}

fn align_to(ptr: u64, align: u32) -> u64 {
    ptr.div_ceil(align as u64) * align as u64
}

/// The alignment of a value of `ty` in memory.
fn alignment(ty: &ValType) -> Result<u32> {
    Ok(match ty {
        ValType::Bool | ValType::S8 | ValType::U8 => 1,
        ValType::S16 | ValType::U16 => 2,
        ValType::S32 | ValType::U32 | ValType::F32 | ValType::Char | ValType::String => 4,
        ValType::S64 | ValType::U64 | ValType::F64 => 8,
        ValType::Defined(d) => match &**d {
            DefinedType::List(_) => 4,
            DefinedType::Flags(labels) => flags_size(labels.len()),
            DefinedType::Own(_) | DefinedType::Borrow(_) => return Err(unsupported("resources")),
            _ => {
                if let Some(fields) = record_fields(d) {
                    let mut a = 1;
                    for t in fields {
                        a = a.max(alignment(t)?);
                    }
                    a
                } else {
                    let cases = despecialized_cases(d).expect("a variant-like type");
                    discriminant_size(cases.len()).max(max_case_alignment(&cases)?)
                }
            }
        },
    })
}

fn max_case_alignment(cases: &[(&str, Option<&ValType>)]) -> Result<u32> {
    let mut a = 1;
    for (_, t) in cases {
        if let Some(t) = t {
            a = a.max(alignment(t)?);
        }
    }
    Ok(a)
}

/// The number of bytes a value of `ty` takes as a list element.
fn elem_size(ty: &ValType) -> Result<u32> {
    Ok(match ty {
        ValType::Bool | ValType::S8 | ValType::U8 => 1,
        ValType::S16 | ValType::U16 => 2,
        ValType::S32 | ValType::U32 | ValType::F32 | ValType::Char => 4,
        ValType::S64 | ValType::U64 | ValType::F64 | ValType::String => 8,
        ValType::Defined(d) => match &**d {
            DefinedType::List(_) => 8,
            DefinedType::Flags(labels) => flags_size(labels.len()),
            DefinedType::Own(_) | DefinedType::Borrow(_) => return Err(unsupported("resources")),
            _ => {
                if let Some(fields) = record_fields(d) {
                    let mut s = 0u64;
                    for t in &fields {
                        s = align_to(s, alignment(t)?) + elem_size(t)? as u64;
                    }
                    align_to(s, alignment(ty)?) as u32
                } else {
                    let cases = despecialized_cases(d).expect("a variant-like type");
                    let mut s = align_to(
                        discriminant_size(cases.len()) as u64,
                        max_case_alignment(&cases)?,
                    );
                    let mut payload = 0;
                    for (_, t) in &cases {
                        if let Some(t) = t {
                            payload = payload.max(elem_size(t)?);
                        }
                    }
                    s += payload as u64;
                    align_to(s, alignment(ty)?) as u32
                }
            }
        },
    })
}

/// Appends the core types a value of `ty` takes when it is passed in registers.
fn flatten(ty: &ValType, out: &mut Vec<ValueType>) -> Result<()> {
    match ty {
        ValType::Bool
        | ValType::S8
        | ValType::U8
        | ValType::S16
        | ValType::U16
        | ValType::S32
        | ValType::U32
        | ValType::Char => out.push(I32),
        ValType::S64 | ValType::U64 => out.push(I64),
        ValType::F32 => out.push(F32),
        ValType::F64 => out.push(F64),
        ValType::String => out.extend([I32, I32]),
        ValType::Defined(d) => match &**d {
            DefinedType::List(_) => out.extend([I32, I32]),
            DefinedType::Flags(_) => out.push(I32),
            DefinedType::Own(_) | DefinedType::Borrow(_) => return Err(unsupported("resources")),
            _ => {
                if let Some(fields) = record_fields(d) {
                    for t in fields {
                        flatten(t, out)?;
                    }
                } else {
                    let cases = despecialized_cases(d).expect("a variant-like type");
                    out.push(I32);
                    out.extend(flatten_payload(&cases)?);
                }
            }
        },
    }
    Ok(())
}

/// The joined core types of a variant's payloads, without the discriminant.
fn flatten_payload(cases: &[(&str, Option<&ValType>)]) -> Result<Vec<ValueType>> {
    let mut flat: Vec<ValueType> = Vec::new();
    for (_, t) in cases {
        if let Some(t) = t {
            let mut case_flat = Vec::new();
            flatten(t, &mut case_flat)?;
            for (i, ft) in case_flat.into_iter().enumerate() {
                if i < flat.len() {
                    flat[i] = join(flat[i], ft);
                } else {
                    flat.push(ft);
                }
            }
        }
    }
    Ok(flat)
}

fn join(a: ValueType, b: ValueType) -> ValueType {
    if a == b {
        a
    } else if (a == I32 && b == F32) || (a == F32 && b == I32) {
        I32
    } else {
        I64
    }
}

fn flat_count(types: &[ValType]) -> Result<usize> {
    let mut out = Vec::new();
    for t in types {
        flatten(t, &mut out)?;
    }
    Ok(out.len())
}

#[derive(Clone, Copy)]
pub enum Context {
    Lift,
    Lower,
}

/// The core type of a function with the component type `ty`, in the synchronous ABI.
pub fn flatten_functype(ty: &FuncType, context: Context) -> Result<CoreFuncType> {
    let mut params = Vec::new();
    for (_, t) in &ty.params {
        flatten(t, &mut params)?;
    }
    let mut results = Vec::new();
    if let Some(t) = &ty.result {
        flatten(t, &mut results)?;
    }
    if params.len() > MAX_FLAT_PARAMS {
        params = vec![I32];
    }
    if results.len() > MAX_FLAT_RESULTS {
        match context {
            Context::Lift => results = vec![I32],
            Context::Lower => {
                params.push(I32);
                results = Vec::new();
            }
        }
    }
    Ok(CoreFuncType { params, results })
}

fn tuple_type(types: &[ValType]) -> ValType {
    ValType::Defined(Shared::new(DefinedType::Tuple(types.to_vec())))
}

fn canonical_nan32(x: f32) -> f32 {
    if x.is_nan() {
        f32::from_bits(0x7fc0_0000)
    } else {
        x
    }
}

fn canonical_nan64(x: f64) -> f64 {
    if x.is_nan() {
        f64::from_bits(0x7ff8_0000_0000_0000)
    } else {
        x
    }
}

fn char_from_i32(i: i32) -> Result<char> {
    char::from_u32(i as u32).ok_or(RuntimeError::InvalidCharBitPattern)
}

fn unpack_flags(bits: u32, labels: &[String]) -> ComponentVal {
    ComponentVal::Flags(
        labels
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < 32 && (bits >> i) & 1 == 1)
            .map(|(_, name)| name.clone())
            .collect(),
    )
}

fn pack_flags(set: &[String], labels: &[String]) -> Result<u32> {
    let mut bits = 0u32;
    for name in set {
        let i = labels
            .iter()
            .position(|l| l == name)
            .ok_or_else(|| trap(format!("unknown flag {name}")))?;
        bits |= 1 << i;
    }
    Ok(bits)
}

fn read_int(cx: &CanonContext, ptr: u32, size: u32) -> Result<u64> {
    let bytes = cx.read(ptr, size)?;
    let mut v = 0u64;
    for (i, b) in bytes.iter().enumerate() {
        v |= (*b as u64) << (8 * i);
    }
    Ok(v)
}

fn write_int(cx: &CanonContext, ptr: u32, size: u32, v: u64) -> Result<()> {
    let bytes: Vec<u8> = (0..size).map(|i| (v >> (8 * i)) as u8).collect();
    cx.write(ptr, &bytes)
}

/// Reads a value of `ty` at `ptr`, which the caller has checked to be aligned and in bounds.
fn load(cx: &CanonContext, ptr: u32, ty: &ValType) -> Result<ComponentVal> {
    Ok(match ty {
        ValType::Bool => ComponentVal::Bool(read_int(cx, ptr, 1)? != 0),
        ValType::S8 => ComponentVal::S8(read_int(cx, ptr, 1)? as i8),
        ValType::U8 => ComponentVal::U8(read_int(cx, ptr, 1)? as u8),
        ValType::S16 => ComponentVal::S16(read_int(cx, ptr, 2)? as i16),
        ValType::U16 => ComponentVal::U16(read_int(cx, ptr, 2)? as u16),
        ValType::S32 => ComponentVal::S32(read_int(cx, ptr, 4)? as i32),
        ValType::U32 => ComponentVal::U32(read_int(cx, ptr, 4)? as u32),
        ValType::S64 => ComponentVal::S64(read_int(cx, ptr, 8)? as i64),
        ValType::U64 => ComponentVal::U64(read_int(cx, ptr, 8)?),
        ValType::F32 => ComponentVal::F32(canonical_nan32(f32::from_bits(
            read_int(cx, ptr, 4)? as u32
        ))),
        ValType::F64 => ComponentVal::F64(canonical_nan64(f64::from_bits(read_int(cx, ptr, 8)?))),
        ValType::Char => ComponentVal::Char(char_from_i32(read_int(cx, ptr, 4)? as i32)?),
        ValType::String => {
            let begin = read_int(cx, ptr, 4)? as u32;
            let len = read_int(cx, ptr + 4, 4)? as u32;
            load_string_from_range(cx, begin, len)?
        }
        ValType::Defined(d) => match &**d {
            DefinedType::List(elem) => {
                let begin = read_int(cx, ptr, 4)? as u32;
                let len = read_int(cx, ptr + 4, 4)? as u32;
                load_list_from_range(cx, begin, len, elem)?
            }
            DefinedType::Flags(labels) => {
                unpack_flags(read_int(cx, ptr, flags_size(labels.len()))? as u32, labels)
            }
            DefinedType::Record(fields) => {
                let mut at = ptr as u64;
                let mut vals = Vec::with_capacity(fields.len());
                for (label, t) in fields {
                    at = align_to(at, alignment(t)?);
                    vals.push((label.clone(), load(cx, at as u32, t)?));
                    at += elem_size(t)? as u64;
                }
                ComponentVal::Record(vals)
            }
            DefinedType::Tuple(ts) => {
                let mut at = ptr as u64;
                let mut vals = Vec::with_capacity(ts.len());
                for t in ts {
                    at = align_to(at, alignment(t)?);
                    vals.push(load(cx, at as u32, t)?);
                    at += elem_size(t)? as u64;
                }
                ComponentVal::Tuple(vals)
            }
            DefinedType::Own(_) | DefinedType::Borrow(_) => return Err(unsupported("resources")),
            _ => {
                let cases = despecialized_cases(d).expect("a variant-like type");
                let disc_size = discriminant_size(cases.len());
                let index = read_int(cx, ptr, disc_size)? as usize;
                if index >= cases.len() {
                    return Err(trap("invalid variant discriminant"));
                }
                let (label, payload_ty) = cases[index];
                let at = align_to(ptr as u64 + disc_size as u64, max_case_alignment(&cases)?);
                let payload = match payload_ty {
                    Some(t) => Some(Box::new(load(cx, at as u32, t)?)),
                    None => None,
                };
                case_value(d, label, payload)
            }
        },
    })
}

/// Writes `val` of `ty` at `ptr`, which the caller has checked to be aligned and in bounds.
fn store(cx: &CanonContext, val: &ComponentVal, ty: &ValType, ptr: u32) -> Result<()> {
    match (ty, val) {
        (ValType::Bool, ComponentVal::Bool(v)) => write_int(cx, ptr, 1, *v as u64),
        (ValType::S8, ComponentVal::S8(v)) => write_int(cx, ptr, 1, *v as u8 as u64),
        (ValType::U8, ComponentVal::U8(v)) => write_int(cx, ptr, 1, *v as u64),
        (ValType::S16, ComponentVal::S16(v)) => write_int(cx, ptr, 2, *v as u16 as u64),
        (ValType::U16, ComponentVal::U16(v)) => write_int(cx, ptr, 2, *v as u64),
        (ValType::S32, ComponentVal::S32(v)) => write_int(cx, ptr, 4, *v as u32 as u64),
        (ValType::U32, ComponentVal::U32(v)) => write_int(cx, ptr, 4, *v as u64),
        (ValType::S64, ComponentVal::S64(v)) => write_int(cx, ptr, 8, *v as u64),
        (ValType::U64, ComponentVal::U64(v)) => write_int(cx, ptr, 8, *v),
        (ValType::F32, ComponentVal::F32(v)) => {
            write_int(cx, ptr, 4, canonical_nan32(*v).to_bits() as u64)
        }
        (ValType::F64, ComponentVal::F64(v)) => {
            write_int(cx, ptr, 8, canonical_nan64(*v).to_bits())
        }
        (ValType::Char, ComponentVal::Char(c)) => write_int(cx, ptr, 4, *c as u64),
        (ValType::String, ComponentVal::String(s)) => {
            let (begin, len) = store_string_into_range(cx, s)?;
            write_int(cx, ptr, 4, begin as u64)?;
            write_int(cx, ptr + 4, 4, len as u64)
        }
        (ValType::Defined(d), val) => match (&**d, val) {
            (DefinedType::List(elem), ComponentVal::List(items)) => {
                let (begin, len) = store_list_into_range(cx, items, elem)?;
                write_int(cx, ptr, 4, begin as u64)?;
                write_int(cx, ptr + 4, 4, len as u64)
            }
            (DefinedType::Flags(labels), ComponentVal::Flags(set)) => write_int(
                cx,
                ptr,
                flags_size(labels.len()),
                pack_flags(set, labels)? as u64,
            ),
            (DefinedType::Record(fields), ComponentVal::Record(vals)) => {
                let mut at = ptr as u64;
                for (label, t) in fields {
                    let v = record_field(vals, label)?;
                    at = align_to(at, alignment(t)?);
                    store(cx, v, t, at as u32)?;
                    at += elem_size(t)? as u64;
                }
                Ok(())
            }
            (DefinedType::Tuple(ts), ComponentVal::Tuple(vals)) => {
                if ts.len() != vals.len() {
                    return Err(RuntimeError::TypeMismatch);
                }
                let mut at = ptr as u64;
                for (t, v) in ts.iter().zip(vals) {
                    at = align_to(at, alignment(t)?);
                    store(cx, v, t, at as u32)?;
                    at += elem_size(t)? as u64;
                }
                Ok(())
            }
            (DefinedType::Own(_) | DefinedType::Borrow(_), _) => Err(unsupported("resources")),
            (d, val) => {
                let cases = despecialized_cases(d).ok_or(RuntimeError::TypeMismatch)?;
                let (index, payload) = case_of(d, val)?;
                let disc_size = discriminant_size(cases.len());
                write_int(cx, ptr, disc_size, index as u64)?;
                let at = align_to(ptr as u64 + disc_size as u64, max_case_alignment(&cases)?);
                match (cases[index].1, payload) {
                    (Some(t), Some(v)) => store(cx, v, t, at as u32),
                    (None, None) => Ok(()),
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
        },
        _ => Err(RuntimeError::TypeMismatch),
    }
}

fn record_field<'v>(vals: &'v [(String, ComponentVal)], label: &str) -> Result<&'v ComponentVal> {
    vals.iter()
        .find(|(l, _)| l == label)
        .map(|(_, v)| v)
        .ok_or_else(|| trap(format!("missing record field {label}")))
}

fn load_string_from_range(cx: &CanonContext, ptr: u32, len: u32) -> Result<ComponentVal> {
    if cx.encoding != StringEncoding::Utf8 {
        return Err(unsupported("string transcoding"));
    }
    if len as u64 > MAX_STRING_BYTE_LENGTH {
        return Err(trap("string content out-of-bounds"));
    }
    let bytes = cx.read(ptr, len)?;
    String::from_utf8(bytes)
        .map(ComponentVal::String)
        .map_err(|_| trap("invalid utf-8"))
}

/// Writes `s` into newly allocated memory and returns the pointer and the length in bytes.
fn store_string_into_range(cx: &CanonContext, s: &str) -> Result<(u32, u32)> {
    if cx.encoding != StringEncoding::Utf8 {
        return Err(unsupported("string transcoding"));
    }
    let len = s.len() as u32;
    let ptr = cx.allocate(1, len)?;
    cx.write(ptr, s.as_bytes())?;
    Ok((ptr, len))
}

fn load_list_from_range(
    cx: &CanonContext,
    ptr: u32,
    len: u32,
    elem: &ValType,
) -> Result<ComponentVal> {
    let size = elem_size(elem)?;
    let byte_len = len as u64 * size as u64;
    if byte_len > MAX_LIST_BYTE_LENGTH {
        return Err(trap("list content out-of-bounds"));
    }
    if !ptr.is_multiple_of(alignment(elem)?) {
        return Err(trap("unaligned pointer"));
    }
    cx.check_range(ptr, byte_len)?;
    let mut items = Vec::with_capacity(len as usize);
    for i in 0..len {
        items.push(load(cx, ptr + i * size, elem)?);
    }
    Ok(ComponentVal::List(items))
}

/// Writes `items` into newly allocated memory and returns the pointer and the element count.
fn store_list_into_range(
    cx: &CanonContext,
    items: &[ComponentVal],
    elem: &ValType,
) -> Result<(u32, u32)> {
    let size = elem_size(elem)?;
    let byte_len = items.len() as u64 * size as u64;
    if byte_len > MAX_LIST_BYTE_LENGTH {
        return Err(trap("list content out-of-bounds"));
    }
    let ptr = cx.allocate(alignment(elem)?, byte_len as u32)?;
    for (i, item) in items.iter().enumerate() {
        store(cx, item, elem, ptr + i as u32 * size)?;
    }
    Ok((ptr, items.len() as u32))
}

/// A source of core values, read in the order the flat types expect them.
pub trait Source {
    fn next(&mut self, want: ValueType) -> Result<Val>;
}

/// Reads core values from a slice.
pub struct Reader<'a> {
    vals: &'a [Val],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(vals: &'a [Val]) -> Self {
        Reader { vals, at: 0 }
    }
}

impl Source for Reader<'_> {
    fn next(&mut self, want: ValueType) -> Result<Val> {
        let v = self
            .vals
            .get(self.at)
            .ok_or_else(|| trap("too few core values"))?;
        self.at += 1;
        if v.val_type() != want {
            return Err(RuntimeError::TypeMismatch);
        }
        Ok(v.clone())
    }
}

fn next_u32(src: &mut dyn Source) -> Result<u32> {
    Ok(src.next(I32)?.to_i32()? as u32)
}

/// Reads a variant payload through the joined types, converting each value to the type the case wants.
struct Coerce<'a> {
    inner: &'a mut dyn Source,
    have: &'a [ValueType],
    at: usize,
}

impl Source for Coerce<'_> {
    fn next(&mut self, want: ValueType) -> Result<Val> {
        let have = *self
            .have
            .get(self.at)
            .ok_or_else(|| trap("too few core values"))?;
        self.at += 1;
        let x = self.inner.next(have)?;
        Ok(match (have, want) {
            (h, w) if h == w => x,
            (I32, F32) => Val::Num(Num::F32(canonical_nan32(
                f32::from_bits(x.to_i32()? as u32),
            ))),
            (I64, I32) => Val::Num(Num::I32(x.to_i64()? as i32)),
            (I64, F32) => Val::Num(Num::F32(canonical_nan32(
                f32::from_bits(x.to_i64()? as u32),
            ))),
            (I64, F64) => Val::Num(Num::F64(canonical_nan64(
                f64::from_bits(x.to_i64()? as u64),
            ))),
            _ => return Err(RuntimeError::TypeMismatch),
        })
    }
}

/// Lifts one value of `ty` from core values.
fn lift_flat(cx: &CanonContext, src: &mut dyn Source, ty: &ValType) -> Result<ComponentVal> {
    Ok(match ty {
        ValType::Bool => ComponentVal::Bool(src.next(I32)?.to_i32()? != 0),
        ValType::S8 => ComponentVal::S8(src.next(I32)?.to_i32()? as i8),
        ValType::U8 => ComponentVal::U8(src.next(I32)?.to_i32()? as u8),
        ValType::S16 => ComponentVal::S16(src.next(I32)?.to_i32()? as i16),
        ValType::U16 => ComponentVal::U16(src.next(I32)?.to_i32()? as u16),
        ValType::S32 => ComponentVal::S32(src.next(I32)?.to_i32()?),
        ValType::U32 => ComponentVal::U32(src.next(I32)?.to_i32()? as u32),
        ValType::S64 => ComponentVal::S64(src.next(I64)?.to_i64()?),
        ValType::U64 => ComponentVal::U64(src.next(I64)?.to_i64()? as u64),
        ValType::F32 => ComponentVal::F32(canonical_nan32(src.next(F32)?.to_f32()?)),
        ValType::F64 => ComponentVal::F64(canonical_nan64(src.next(F64)?.to_f64()?)),
        ValType::Char => ComponentVal::Char(char_from_i32(src.next(I32)?.to_i32()?)?),
        ValType::String => {
            let ptr = next_u32(src)?;
            let len = next_u32(src)?;
            load_string_from_range(cx, ptr, len)?
        }
        ValType::Defined(d) => match &**d {
            DefinedType::List(elem) => {
                let ptr = next_u32(src)?;
                let len = next_u32(src)?;
                load_list_from_range(cx, ptr, len, elem)?
            }
            DefinedType::Flags(labels) => unpack_flags(next_u32(src)?, labels),
            DefinedType::Record(fields) => {
                let mut vals = Vec::with_capacity(fields.len());
                for (label, t) in fields {
                    vals.push((label.clone(), lift_flat(cx, src, t)?));
                }
                ComponentVal::Record(vals)
            }
            DefinedType::Tuple(ts) => {
                let mut vals = Vec::with_capacity(ts.len());
                for t in ts {
                    vals.push(lift_flat(cx, src, t)?);
                }
                ComponentVal::Tuple(vals)
            }
            DefinedType::Own(_) | DefinedType::Borrow(_) => return Err(unsupported("resources")),
            _ => {
                let cases = despecialized_cases(d).expect("a variant-like type");
                let flat = flatten_payload(&cases)?;
                let index = next_u32(src)? as usize;
                if index >= cases.len() {
                    return Err(trap("invalid variant discriminant"));
                }
                let (label, payload_ty) = cases[index];
                let (payload, consumed) = {
                    let mut coerce = Coerce {
                        inner: src,
                        have: &flat,
                        at: 0,
                    };
                    let payload = match payload_ty {
                        Some(t) => Some(Box::new(lift_flat(cx, &mut coerce, t)?)),
                        None => None,
                    };
                    (payload, coerce.at)
                };
                for have in &flat[consumed..] {
                    src.next(*have)?;
                }
                case_value(d, label, payload)
            }
        },
    })
}

/// Lowers one value of `ty` into core values appended to `out`.
fn lower_flat(
    cx: &CanonContext,
    val: &ComponentVal,
    ty: &ValType,
    out: &mut Vec<Val>,
) -> Result<()> {
    let i32_val = |v: i32| Val::Num(Num::I32(v));
    match (ty, val) {
        (ValType::Bool, ComponentVal::Bool(v)) => out.push(i32_val(*v as i32)),
        (ValType::S8, ComponentVal::S8(v)) => out.push(i32_val(*v as i32)),
        (ValType::U8, ComponentVal::U8(v)) => out.push(i32_val(*v as i32)),
        (ValType::S16, ComponentVal::S16(v)) => out.push(i32_val(*v as i32)),
        (ValType::U16, ComponentVal::U16(v)) => out.push(i32_val(*v as i32)),
        (ValType::S32, ComponentVal::S32(v)) => out.push(i32_val(*v)),
        (ValType::U32, ComponentVal::U32(v)) => out.push(i32_val(*v as i32)),
        (ValType::S64, ComponentVal::S64(v)) => out.push(Val::Num(Num::I64(*v))),
        (ValType::U64, ComponentVal::U64(v)) => out.push(Val::Num(Num::I64(*v as i64))),
        (ValType::F32, ComponentVal::F32(v)) => out.push(Val::Num(Num::F32(canonical_nan32(*v)))),
        (ValType::F64, ComponentVal::F64(v)) => out.push(Val::Num(Num::F64(canonical_nan64(*v)))),
        (ValType::Char, ComponentVal::Char(c)) => out.push(i32_val(*c as i32)),
        (ValType::String, ComponentVal::String(s)) => {
            let (ptr, len) = store_string_into_range(cx, s)?;
            out.extend([i32_val(ptr as i32), i32_val(len as i32)]);
        }
        (ValType::Defined(d), val) => match (&**d, val) {
            (DefinedType::List(elem), ComponentVal::List(items)) => {
                let (ptr, len) = store_list_into_range(cx, items, elem)?;
                out.extend([i32_val(ptr as i32), i32_val(len as i32)]);
            }
            (DefinedType::Flags(labels), ComponentVal::Flags(set)) => {
                out.push(i32_val(pack_flags(set, labels)? as i32))
            }
            (DefinedType::Record(fields), ComponentVal::Record(vals)) => {
                for (label, t) in fields {
                    lower_flat(cx, record_field(vals, label)?, t, out)?;
                }
            }
            (DefinedType::Tuple(ts), ComponentVal::Tuple(vals)) => {
                if ts.len() != vals.len() {
                    return Err(RuntimeError::TypeMismatch);
                }
                for (t, v) in ts.iter().zip(vals) {
                    lower_flat(cx, v, t, out)?;
                }
            }
            (DefinedType::Own(_) | DefinedType::Borrow(_), _) => {
                return Err(unsupported("resources"))
            }
            (d, val) => {
                let cases = despecialized_cases(d).ok_or(RuntimeError::TypeMismatch)?;
                let flat = flatten_payload(&cases)?;
                let (index, payload) = case_of(d, val)?;
                out.push(i32_val(index as i32));
                let mut payload_vals = Vec::new();
                match (cases[index].1, payload) {
                    (Some(t), Some(v)) => lower_flat(cx, v, t, &mut payload_vals)?,
                    (None, None) => {}
                    _ => return Err(RuntimeError::TypeMismatch),
                }
                let mut wants = flat.iter();
                for v in payload_vals {
                    let want = *wants.next().ok_or(RuntimeError::TypeMismatch)?;
                    out.push(match (v.val_type(), want) {
                        (h, w) if h == w => v,
                        (F32, I32) => i32_val(v.to_f32()?.to_bits() as i32),
                        (I32, I64) => Val::Num(Num::I64(v.to_i32()? as u32 as i64)),
                        (F32, I64) => Val::Num(Num::I64(v.to_f32()?.to_bits() as i64)),
                        (F64, I64) => Val::Num(Num::I64(v.to_f64()?.to_bits() as i64)),
                        _ => return Err(RuntimeError::TypeMismatch),
                    });
                }
                for want in wants {
                    out.push(match *want {
                        I64 => Val::Num(Num::I64(0)),
                        F32 => Val::Num(Num::F32(0.0)),
                        F64 => Val::Num(Num::F64(0.0)),
                        _ => i32_val(0),
                    });
                }
            }
        },
        _ => return Err(RuntimeError::TypeMismatch),
    }
    Ok(())
}

pub fn lift_values(
    cx: &CanonContext,
    src: &mut dyn Source,
    types: &[ValType],
    max_flat: usize,
) -> Result<Vec<ComponentVal>> {
    if flat_count(types)? > max_flat {
        let tuple = tuple_type(types);
        let ptr = next_u32(src)?;
        if !ptr.is_multiple_of(alignment(&tuple)?) {
            return Err(trap("unaligned pointer"));
        }
        cx.check_range(ptr, elem_size(&tuple)? as u64)?;
        match load(cx, ptr, &tuple)? {
            ComponentVal::Tuple(vals) => Ok(vals),
            _ => unreachable!("a tuple type loads a tuple"),
        }
    } else {
        types.iter().map(|t| lift_flat(cx, src, t)).collect()
    }
}

pub fn lower_values(
    cx: &CanonContext,
    vals: &[ComponentVal],
    types: &[ValType],
    max_flat: usize,
    out_param: Option<&mut dyn Source>,
) -> Result<Vec<Val>> {
    if vals.len() != types.len() {
        return Err(RuntimeError::InvalidParameterCount);
    }
    if flat_count(types)? > max_flat {
        let tuple = tuple_type(types);
        let align = alignment(&tuple)?;
        let size = elem_size(&tuple)?;
        let (ptr, flat) = match out_param {
            None => {
                let ptr = cx.allocate(align, size)?;
                (ptr, vec![Val::Num(Num::I32(ptr as i32))])
            }
            Some(src) => {
                let ptr = next_u32(src)?;
                if !ptr.is_multiple_of(align) {
                    return Err(trap("unaligned pointer"));
                }
                cx.check_range(ptr, size as u64)?;
                (ptr, Vec::new())
            }
        };
        store(cx, &ComponentVal::Tuple(vals.to_vec()), &tuple, ptr)?;
        Ok(flat)
    } else {
        let mut out = Vec::new();
        for (v, t) in vals.iter().zip(types) {
            lower_flat(cx, v, t, &mut out)?;
        }
        Ok(out)
    }
}
