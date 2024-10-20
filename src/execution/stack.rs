use super::{value::*, module::*, func::*};
use crate::structure::instructions::Instr;
use std::rc::Weak;

pub struct Stacks {
    pub valueStack: Vec<Val>,
    pub activationFrameStack: Vec<Frame>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Stacks{
        Stacks{
            valueStack: params.clone(),
            activationFrameStack: vec![
                Frame{
                    locals: Vec::new(),
                    module: Weak::new(),
                    labelStack: vec![
                        Label{
                            instrs: vec![
                                AdminInstr::Invoke(funcaddr.clone())
                            ],
                        }
                    ],
                }
            ],
        }
    }
    pub fn exec_instr(&mut self){
        let mut cur_frame = self.activationFrameStack.last_mut().unwrap();
        let mut cur_label = cur_frame.labelStack.last_mut().unwrap();
    }
}

pub struct Frame {
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
    pub labelStack: Vec<Label>,
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
