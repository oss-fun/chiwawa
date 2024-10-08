use super::{value::*, module::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    labels: Vec<LabelStack>,
    activationFrames: Vec<FrameStack>,
}

struct FrameStack {
    pub locals:Vec<Val>,
    pub module: Weak<ModuleInst>,
}

struct LabelStack {
    pub instrs: Vec<Instr>
}