use super::{value::*, module::*, func::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    valueStack: Vec<Val>,
    labelsStack: Vec<Label>,
    activationFrameStack: Vec<Frame>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Stacks{
        Stacks{
            valueStack: params.clone(),
            labelsStack: vec![
                Label{
                    instrs: vec![
                        AdminInstr::Invoke(funcaddr.clone())
                    ],
                }
            ],
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
    pub instrs: Vec<AdminInstr>
}

pub enum AdminInstr {
    Trap,
    Ref(FuncAddr),
    RefExtern(ExternAddr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Frame(Frame, Vec<Instr>),
}