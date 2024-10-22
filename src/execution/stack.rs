use super::{value::*, module::*, func::*};
use crate::structure::{instructions::Instr, types::*};
use crate::error::RuntimeError;
use std::rc::Weak;
use std::borrow::Borrow;

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
                    labelStack: vec![
                        Label{
                            instrs: vec![
                                AdminInstr::FrameAdminInstr(FrameAdminInstr::Invoke(funcaddr.clone()))
                            ],
                            valueStack: params.clone(),
                        },
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
                    match &*func_addr.borrow(){
                        FuncInst::RuntimeFunc{type_,module,code} => {
                            let frame = Frame{
                                locals: {
                                    let mut locals = Vec::new();
                                    locals.append(
                                        &mut cur_label.valueStack.split_off(cur_label.valueStack.len() - type_.params.len())
                                    );
                                    locals.append(
                                        &mut code.locals.iter().map(|v| match v.1{
                                            ValueType::NumType(NumType::I32) => Val::Num(Num::I32(v.0 as i32)),
                                            ValueType::NumType(NumType::I64) => Val::Num(Num::I64(v.0 as i64)),
                                            ValueType::NumType(NumType::F32) => Val::Num(Num::F32(v.0)),
                                            ValueType::NumType(NumType::F64) => Val::Num(Num::F64(v.0 as u64)),
                                            ValueType::VecType(_) | ValueType::RefType(_) => todo!(),
                                        }).collect()
                                    );
                                    locals
                                },
                                module: module.clone(),
                                labelStack: vec![
                                    Label{
                                        instrs: code.body.0.clone().into_iter().map(AdminInstr::Instr).collect(),
                                        valueStack: vec![],
                                    }
                                ],
                                void:type_.results.iter().count() ==0 ,                           
                            };
                            self.activationFrameStack.push(frame);

                        },
                        FuncInst::HostFunc{..} => {
                            todo!()
                        },
                    }

                },
                FrameAdminInstr::Return =>{
                    let ret = cur_label.valueStack.pop();
                    if !self.activationFrameStack.pop().unwrap().void{
                        let mut next = self.activationFrameStack.last_mut().unwrap();
                        next.labelStack.last_mut().unwrap().valueStack.push(ret.unwrap());
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
    pub void: bool,
}

impl Frame{
    pub fn exec_instr_frame_level(&mut self) -> Result<Option<FrameAdminInstr>, RuntimeError>{
        let mut cur_label = self.labelStack.last_mut().unwrap();
        Ok(None)
    }
}
pub struct Label {
    pub instrs: Vec<AdminInstr>,
    pub valueStack: Vec<Val>,
}

pub enum FrameAdminInstr{
    Invoke(FuncAddr),
    Return,
}

pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Ref(FuncAddr),
    FrameAdminInstr(FrameAdminInstr),
    RefExtern(ExternAddr),
    Label(Label, Vec<Instr>),
    Frame(Frame, Vec<Instr>),
}