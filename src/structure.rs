//! Internal representation of WebAssembly modules and types.
//!
//! This module defines the data structures that represent parsed WebAssembly
//! modules before and after preprocessing.
//!
//! ## Module Organization
//!
//! - [`instructions`]: WebAssembly instruction set representation
//! - [`module`]: Module structure including functions, memory, tables, and globals
//! - [`types`]: WebAssembly type definitions (value types, function types, etc.)

pub mod instructions;
pub mod module;
pub mod types;
