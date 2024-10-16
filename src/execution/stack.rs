use super::{value::*, module::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    valueStack: Vec<Val>,
    labelsStack: Vec<Label>,
    activationFrameStack: Vec<Frame>,
}

impl Stacks {
    pub fn new(params: Vec<Val>) -> Stacks{
        Stacks{
            valueStack: params.clone(),
            labelsStack: Vec::new(),
            activationFrameStack: vec![
                Frame{
                    locals: Vec::new(),
                    module: Weak::new()
                }
            ],
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