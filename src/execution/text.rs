//! The text form of component values, as `wast` scripts write them: `(s32.const 5)`, `(str.const "a")`.

use super::canon::ComponentVal;
use std::fmt;
use wast::component::WastVal;
use wast::parser::{self, Parse, ParseBuffer, Parser};

struct Values<'a>(Vec<WastVal<'a>>);

impl<'a> Parse<'a> for Values<'a> {
    fn parse(parser: Parser<'a>) -> parser::Result<Self> {
        let mut vals = Vec::new();
        while !parser.is_empty() {
            vals.push(parser.parens(|p| p.parse())?);
        }
        Ok(Values(vals))
    }
}

/// Parses a sequence of parenthesized component values.
pub fn parse_values(text: &str) -> parser::Result<Vec<ComponentVal>> {
    let buf = ParseBuffer::new(text)?;
    let Values(vals) = parser::parse::<Values>(&buf)?;
    Ok(vals.iter().map(from_wast).collect())
}

pub fn from_wast(val: &WastVal) -> ComponentVal {
    let boxed = |v: &Option<Box<WastVal>>| v.as_deref().map(from_wast).map(Box::new);
    match val {
        WastVal::Bool(v) => ComponentVal::Bool(*v),
        WastVal::S8(v) => ComponentVal::S8(*v),
        WastVal::U8(v) => ComponentVal::U8(*v),
        WastVal::S16(v) => ComponentVal::S16(*v),
        WastVal::U16(v) => ComponentVal::U16(*v),
        WastVal::S32(v) => ComponentVal::S32(*v),
        WastVal::U32(v) => ComponentVal::U32(*v),
        WastVal::S64(v) => ComponentVal::S64(*v),
        WastVal::U64(v) => ComponentVal::U64(*v),
        WastVal::F32(v) => ComponentVal::F32(f32::from_bits(v.bits)),
        WastVal::F64(v) => ComponentVal::F64(f64::from_bits(v.bits)),
        WastVal::Char(c) => ComponentVal::Char(*c),
        WastVal::String(s) => ComponentVal::String(s.to_string()),
        WastVal::List(items) => ComponentVal::List(items.iter().map(from_wast).collect()),
        WastVal::Record(fields) => ComponentVal::Record(
            fields
                .iter()
                .map(|(name, v)| (name.to_string(), from_wast(v)))
                .collect(),
        ),
        WastVal::Tuple(items) => ComponentVal::Tuple(items.iter().map(from_wast).collect()),
        WastVal::Variant(name, payload) => ComponentVal::Variant(name.to_string(), boxed(payload)),
        WastVal::Enum(name) => ComponentVal::Enum(name.to_string()),
        WastVal::Option(v) => ComponentVal::Option(boxed(v)),
        WastVal::Result(r) => ComponentVal::Result(match r {
            Ok(v) => Ok(boxed(v)),
            Err(v) => Err(boxed(v)),
        }),
        WastVal::Flags(names) => {
            ComponentVal::Flags(names.iter().map(|name| name.to_string()).collect())
        }
    }
}

impl fmt::Display for ComponentVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        self.write_inner(f)?;
        write!(f, ")")
    }
}

impl ComponentVal {
    /// Writes the value without its outer parentheses, as a record field is written.
    fn write_inner(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentVal::Bool(v) => write!(f, "bool.const {v}"),
            ComponentVal::S8(v) => write!(f, "s8.const {v}"),
            ComponentVal::U8(v) => write!(f, "u8.const {v}"),
            ComponentVal::S16(v) => write!(f, "s16.const {v}"),
            ComponentVal::U16(v) => write!(f, "u16.const {v}"),
            ComponentVal::S32(v) => write!(f, "s32.const {v}"),
            ComponentVal::U32(v) => write!(f, "u32.const {v}"),
            ComponentVal::S64(v) => write!(f, "s64.const {v}"),
            ComponentVal::U64(v) => write!(f, "u64.const {v}"),
            ComponentVal::F32(v) => write!(f, "f32.const {}", f32_text(*v)),
            ComponentVal::F64(v) => write!(f, "f64.const {}", f64_text(*v)),
            ComponentVal::Char(c) => write!(f, "char.const \"{}\"", escaped(&c.to_string())),
            ComponentVal::String(s) => write!(f, "str.const \"{}\"", escaped(s)),
            ComponentVal::List(items) => sequence(f, "list.const", items),
            ComponentVal::Record(fields) => {
                write!(f, "record.const")?;
                for (name, v) in fields {
                    write!(f, " (field \"{}\" ", escaped(name))?;
                    v.write_inner(f)?;
                    write!(f, ")")?;
                }
                Ok(())
            }
            ComponentVal::Tuple(items) => sequence(f, "tuple.const", items),
            ComponentVal::Variant(name, payload) => {
                write!(f, "variant.const \"{}\"", escaped(name))?;
                payload_text(f, payload)
            }
            ComponentVal::Enum(name) => write!(f, "enum.const \"{}\"", escaped(name)),
            ComponentVal::Option(None) => write!(f, "option.none"),
            ComponentVal::Option(Some(v)) => write!(f, "option.some {v}"),
            ComponentVal::Result(Ok(v)) => {
                write!(f, "result.ok")?;
                payload_text(f, v)
            }
            ComponentVal::Result(Err(v)) => {
                write!(f, "result.err")?;
                payload_text(f, v)
            }
            ComponentVal::Flags(names) => {
                write!(f, "flags.const")?;
                for name in names {
                    write!(f, " \"{}\"", escaped(name))?;
                }
                Ok(())
            }
            ComponentVal::Resource(rep) => write!(f, "resource.const {rep}"),
        }
    }
}

fn sequence(f: &mut fmt::Formatter<'_>, keyword: &str, items: &[ComponentVal]) -> fmt::Result {
    write!(f, "{keyword}")?;
    for item in items {
        write!(f, " {item}")?;
    }
    Ok(())
}

fn payload_text(f: &mut fmt::Formatter<'_>, payload: &Option<Box<ComponentVal>>) -> fmt::Result {
    match payload {
        Some(v) => write!(f, " {v}"),
        None => Ok(()),
    }
}

/// Escapes a string for a `wast` string literal.
fn escaped(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn f32_text(v: f32) -> String {
    let payload = (v.to_bits() & 0x7f_ffff) as u64;
    float_text(v, payload, 0x40_0000, || format!("{v:?}"))
}

fn f64_text(v: f64) -> String {
    let payload = v.to_bits() & 0xf_ffff_ffff_ffff;
    float_text(v, payload, 0x8_0000_0000_0000, || format!("{v:?}"))
}

/// Writes a NaN as `nan` or `nan:0x<payload>`, an infinity as `inf`, and any other value in decimal.
fn float_text<F: Into<f64> + Copy>(
    v: F,
    payload: u64,
    canonical: u64,
    finite: impl Fn() -> String,
) -> String {
    let v: f64 = v.into();
    let sign = if v.is_sign_negative() { "-" } else { "" };
    if v.is_nan() {
        return if payload == canonical {
            format!("{sign}nan")
        } else {
            format!("{sign}nan:0x{payload:x}")
        };
    }
    if v.is_infinite() {
        return format!("{sign}inf");
    }
    finite()
}
