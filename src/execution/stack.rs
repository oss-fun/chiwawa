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
                        LabelStack{
                            label: Label{
                                instrs: vec![],
                            },
                            instrs: vec![
                                AdminInstr::ModuleInstr(ModuleInstr::Invoke(funcaddr.clone()))
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
                ModuleInstr::Invoke(func_addr) => {
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
                                            ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(v.0 as i128)),
                                            ValueType::RefType(_) => todo!(),
                                        }).collect()
                                    );
                                    locals
                                },
                                module: module.clone(),
                                labelStack: vec![
                                    LabelStack{
                                        label: Label{
                                            instrs: vec![],
                                        },
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
                ModuleInstr::Return =>{
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
    pub labelStack: Vec<LabelStack>,
    pub void: bool,
}

impl Frame{
    pub fn exec_instr_frame_level(&mut self) -> Result<Option<ModuleInstr>, RuntimeError>{
        let mut cur_label = self.labelStack.last_mut().unwrap();
        if let Some(instr) = cur_label.exec_primitive_instr()?{
            match instr {
                /*Redirect to Exec_instr(Handing Instruction Spanning Frame)*/
                FrameInstr::ModuleInstr(module_instr) => {
                    match module_instr{
                        ModuleInstr::Return => Ok(Some(ModuleInstr::Return)),
                        ModuleInstr::Invoke(fa) => Ok(Some(ModuleInstr::Invoke(fa))),

                    }
                },
                FrameInstr::Br(_) => todo!(),
                FrameInstr::Label(label, instrs) => {
                    self.labelStack.push(
                        LabelStack{
                            label,
                            instrs: instrs.into_iter().map(AdminInstr::Instr).collect(),
                            valueStack: vec![],
                        }
                    );
                    Ok(None)
                },
                FrameInstr::EndLabel => {
                    let mut cur_label = self.labelStack.pop().unwrap();
                    if let Some(last) = self.labelStack.last_mut() {
                        last.valueStack.append(&mut cur_label.valueStack);
                        Ok(None)
                    } else {
                        self.labelStack.push(cur_label);
                        Ok(Some(ModuleInstr::Return)) 
                    }
                },
            }
        } else {
            Ok(None)
        }
    }
}
pub struct Label {
    pub instrs: Vec<Instr>
}

pub struct LabelStack {
    pub label: Label,
    pub instrs: Vec<AdminInstr>,
    pub valueStack: Vec<Val>,
}

impl LabelStack{
    pub fn exec_primitive_instr(&self) -> Result<Option<FrameInstr>, RuntimeError>{
        Ok(None)
    }
}

pub enum ModuleInstr{
    Invoke(FuncAddr),
    Return,
}

pub enum FrameInstr{
    Br(LabelIdx),
    Label(Label, Vec<Instr>),
    EndLabel,
    ModuleInstr(ModuleInstr)
}

pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Ref(FuncAddr),
    ModuleInstr(ModuleInstr),
    FrameInstr(FrameInstr),
    RefExtern(ExternAddr),
    Frame(Frame, Vec<Instr>),
}