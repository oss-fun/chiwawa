use super::{value::*, module::*, func::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    pub valueStack: Vec<Val>,
    pub labelsStack: Vec<Label>,
    pub activationFrameStack: Vec<Frame>,
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
    pub fn exec_instr(&mut self){
        let mut cur_frame = &self.activationFrameStack.last_mut().unwrap();
        let mut cur_label = &self.labelsStack.last_mut().unwrap();
    }
}

pub struct Frame {
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
}

pub struct Label {
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