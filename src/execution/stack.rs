use super::{value::*, module::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    valueStack: Vec<Val>,
    labelsStack: Vec<Label>,
    activationFrameStack: Vec<Frame>,
}

impl Stacks {
    pub fn new() -> Stacks{
        Stacks{
            valueStack: Vec::new(),
            labelsStack: Vec::new(),
            activationFrameStack: Vec::new(),
        }
    }
}

struct Frame {
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
}

struct Label {
    pub instrs: Vec<Instr>
}