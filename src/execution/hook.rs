//! Hooks let a guest run its own code at fixed points of a Chiwawa run.

use crate::error::RuntimeError;
use crate::execution::func::FuncAddr;
use crate::execution::module::ModuleInst;
use crate::execution::runtime::{Runtime, RuntimeConfig};
use crate::structure::types::FuncType;
use clap::ValueEnum;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

/// A hook point is a place in a Chiwawa run where Chiwawa calls a guest export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum HookPoint {
    /// Chiwawa calls this hook after it restores the state and before a restored thread resumes.
    #[value(name = "after-restore")]
    AfterRestore,
}

impl HookPoint {
    /// Returns the name of the guest export that Chiwawa calls at this point.
    pub fn export_name(self) -> &'static str {
        match self {
            HookPoint::AfterRestore => "chiwawa_after_restore",
        }
    }
}

impl fmt::Display for HookPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("each hook point has a name")
            .get_name()
            .fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum HookError {
    #[error("hook point {0} needs an exported function {} of type () -> ()", .0.export_name())]
    Export(HookPoint),
    #[error("hook point {0} failed: {1}")]
    Failed(HookPoint, RuntimeError),
}

/// Hooks holds the guest function of each hook point that the user requested.
pub struct Hooks {
    inst: Rc<ModuleInst>,
    funcs: Vec<(HookPoint, FuncAddr)>,
}

impl Hooks {
    /// Finds the export of each point in `points` and checks that its type is `() -> ()`.
    pub fn new(points: &[HookPoint], inst: &Rc<ModuleInst>) -> Result<Self, HookError> {
        let funcs = points
            .iter()
            .map(|&point| {
                inst.get_export_func(point.export_name())
                    .ok()
                    .filter(|func| *func.func_type() == FuncType::default())
                    .map(|func| (point, func))
                    .ok_or(HookError::Export(point))
            })
            .collect::<Result<_, _>>()?;
        Ok(Hooks {
            inst: Rc::clone(inst),
            funcs,
        })
    }

    /// Runs the hook of `point` to completion if the user requested `point`.
    pub fn run(&self, point: HookPoint) -> Result<(), HookError> {
        let Some((_, func)) = self.funcs.iter().find(|(p, _)| *p == point) else {
            return Ok(());
        };
        Runtime::new(
            Rc::clone(&self.inst),
            func,
            Vec::new(),
            RuntimeConfig::default(),
        )
        .and_then(|mut runtime| runtime.run())
        .map_err(|e| HookError::Failed(point, e))?;
        Ok(())
    }
}
