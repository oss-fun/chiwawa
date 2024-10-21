use super::{value::*, module::*, func::*};
use crate::structure::instructions::Instr;
use crate::error::RuntimeError;
use std::rc::Weak;

pub struct Stacks {
    pub activationFrameStack: Vec<Frame>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Stacks{
        Stacks{
            activationFrameStack: vec![
                Frame{
                    locals: Vec::new(),
                    module: Weak::new(),
                    valueStack: params.clone(),
                    labelStack: vec![
                        Label{
                            instrs: vec![
                                AdminInstr::FrameAdminInstr(FrameAdminInstr::Invoke(funcaddr.clone()))
                            ],
                        }
                    ],
                    void: true,
                }
            ],
        }
    }

    /*
    This Function Only Handle Instruction Spanning FrameStack.
    i.e., Invoke Wasm Function, Return Function and Call Host-function.
    */
    pub fn exec_instr(&mut self) -> Result<(), RuntimeError>{
        let mut cur_frame = self.activationFrameStack.last_mut().unwrap();
        if let Some(instr) = cur_frame.exec_instr_frame_level()? {
            let mut cur_label = cur_frame.labelStack.last_mut().unwrap();
            match instr {
                FrameAdminInstr::Invoke(func_addr) => {   
                    todo!()
                },
                FrameAdminInstr::Return =>{
                    let ret = cur_frame.valueStack.pop();
                    if !self.activationFrameStack.pop().unwrap().void{
                        let mut next = self.activationFrameStack.last_mut().unwrap();
                        next.valueStack.push(ret.unwrap());
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct Frame {
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
    pub labelStack: Vec<Label>,
    pub valueStack: Vec<Val>,
    pub void: bool,
}

impl Frame{
    pub fn exec_instr_frame_level(&mut self) -> Result<Option<FrameAdminInstr>, RuntimeError>{
        let mut cur_label = self.labelStack.last_mut().unwrap();
        Ok(None)
    }
}
pub struct Label {
    pub instrs: Vec<AdminInstr>
}

pub enum FrameAdminInstr{
    Invoke(FuncAddr),
    Return,
}

pub enum AdminInstr {
    Trap,
    Ref(FuncAddr),
    FrameAdminInstr(FrameAdminInstr),
    RefExtern(ExternAddr),
    Label(Label, Vec<Instr>),
    Frame(Frame, Vec<Instr>),
}