use super::{value::*, module::*, func::*};
use crate::structure::{instructions::Instr, types::*};
use crate::error::RuntimeError;
use std::rc::Weak;
use std::borrow::Borrow;

pub struct Stacks {
    pub activationFrameStack: Vec<FrameStack>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Stacks{
        Stacks{
            activationFrameStack: vec![
                FrameStack{
                    frame: Frame{
                        locals: Vec::new(),
                        module: Weak::new(),
                    },
                    labelStack: vec![
                        LabelStack{
                            label: Label{
                                continue_: vec![],
                                locals_num: 0,
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
                            let frame = FrameStack{
                                frame: Frame{
                                    locals: {
                                        let mut locals = Vec::new();
                                        locals.append(
                                            &mut cur_label.valueStack
                                        );
                                        locals.append(
                                            &mut code.locals.iter().map(|v| match v.1{
                                                ValueType::NumType(NumType::I32) => Val::Num(Num::I32(v.0 as i32)),
                                                ValueType::NumType(NumType::I64) => Val::Num(Num::I64(v.0 as i64)),
                                                ValueType::NumType(NumType::F32) => Val::Num(Num::F32(v.0 as f32)),
                                                ValueType::NumType(NumType::F64) => Val::Num(Num::F64(v.0 as f64)),
                                                ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(v.0 as i128)),
                                                ValueType::RefType(_) => todo!(),
                                            }).collect()
                                        );
                                        locals
                                    },
                                    module: module.clone(),
                                },
                                labelStack: vec![
                                    LabelStack{
                                        label: Label{
                                            continue_: vec![],
                                            locals_num: 0,
                                        },
                                        instrs: code.body.0.clone().into_iter().map(AdminInstr::Instr).rev().collect(),
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

pub struct Frame{
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
}

pub struct FrameStack {
    pub frame: Frame,
    pub labelStack: Vec<LabelStack>,
    pub void: bool,
}

impl FrameStack{
    pub fn exec_instr_frame_level(&mut self) -> Result<Option<ModuleInstr>, RuntimeError>{
        let mut cur_label = self.labelStack.last_mut().unwrap();
        if let Some(instr) = cur_label.exec_primitive_instr(&mut self.frame)?{
            match instr {
                /*Redirect to Exec_instr(Handing Instruction Spanning Frame)*/
                FrameInstr::ModuleInstr(module_instr) => {
                    match module_instr{
                        ModuleInstr::Return => Ok(Some(ModuleInstr::Return)),
                        ModuleInstr::Invoke(fa) => Ok(Some(ModuleInstr::Invoke(fa))),

                    }
                },
                FrameInstr::Br(idx) => {
                    let idx = idx.to_usize();
                    let mut cur_label_value = self.labelStack.last().unwrap().valueStack.clone();
                    for _ in 0..idx{
                        self.labelStack.pop();
                    };
                    
                    let continue_label = self.labelStack.pop().unwrap().label;
                    let mut instrs = continue_label.continue_.into_iter().map(AdminInstr::Instr).rev().collect();

                    if let Some(dst_label) = self.labelStack.last_mut(){
                        dst_label.valueStack.append(&mut cur_label_value);
                        dst_label.instrs.append(&mut instrs);
                        Ok(None)
                    }else{
                        self.labelStack.push(
                            LabelStack{
                                label: Label{
                                    continue_: vec![],
                                    locals_num: 0,
                                },
                                instrs: vec![],
                                valueStack: cur_label_value,
                            }
                        );
                        Ok(Some(ModuleInstr::Return))
                    }
                },
                FrameInstr::Label(label, instrs) => {
                    self.labelStack.push(
                        LabelStack{
                            label,
                            instrs: instrs.into_iter().map(AdminInstr::Instr).rev().collect(),
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

#[derive(Clone)]
pub struct Label {
    pub continue_: Vec<Instr>,
    pub locals_num: usize,
}

pub struct LabelStack {
    pub label: Label,
    pub instrs: Vec<AdminInstr>,
    pub valueStack: Vec<Val>,
}

impl LabelStack{
    pub fn exec_primitive_instr(&mut self, frame: &mut Frame) -> Result<Option<FrameInstr>, RuntimeError>{
        Ok(if let Some(instr) = self.instrs.pop(){
            match instr {
                AdminInstr::Instr(instr) => {
                    match instr {
                        /*Numeric Instructions*/
                        Instr::I32Const(x) => {
                            self.valueStack.push(Val::Num(Num::I32(x)));
                            None
                        },
                        Instr::I64Const(x) => {
                            self.valueStack.push(Val::Num(Num::I64(x)));
                            None
                        },
                        Instr::F32Const(x) => {
                            self.valueStack.push(Val::Num(Num::F32(x as f32)));
                            None
                        },
                        Instr::F64Const(x) => {
                            self.valueStack.push(Val::Num(Num::F64(x as f64)));
                            None
                        },
                        Instr::I32Clz => {
                            let x = self.valueStack.pop().unwrap().to_i32().leading_zeros();
                            self.valueStack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I32Ctz => {
                            let x = self.valueStack.pop().unwrap().to_i32().trailing_zeros();
                            self.valueStack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I32Popcnt => {
                            let x = self.valueStack.pop().unwrap().to_i32().count_ones();
                            self.valueStack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I64Clz => {
                            let x = self.valueStack.pop().unwrap().to_i64().leading_zeros();
                            self.valueStack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::I64Ctz => {
                            let x = self.valueStack.pop().unwrap().to_i64().trailing_zeros();
                            self.valueStack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::I64Popcnt => {
                            let x = self.valueStack.pop().unwrap().to_i64().count_ones();
                            self.valueStack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::F32Abs => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x.abs())));
                            None
                        },
                        Instr::F32Neg => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x * -1.0)));
                            None
                        },
                        Instr::F32Sqrt => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x.sqrt())));
                            None
                        },
                        Instr::F32Ceil => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x.ceil())));
                            None
                        },
                        Instr::F32Floor => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x.floor())));
                            None
                        },
                        Instr::F32Trunc => {
                            let x = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(Val::Num(Num::F32(x.trunc())));
                            None
                        },
                        Instr::F32Nearest => {
                            let x = self.valueStack.pop().unwrap().to_f32() % 2.0;
                            
                            let ret = if x == 0.5 {
                                x.floor()
                            }else if x ==  -0.5{
                                x.ceil()
                            }else{
                                x.round()
                            };
                            self.valueStack.push(Val::Num(Num::F32(ret)));
                            None
                        },
                        Instr::F64Abs => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x.abs())));
                            None
                        },
                        Instr::F32Neg => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x * -1.0)));
                            None
                        },
                        Instr::F32Sqrt => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x.sqrt())));
                            None
                        },
                        Instr::F32Ceil => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x.ceil())));
                            None
                        },
                        Instr::F32Floor => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x.floor())));
                            None
                        },
                        Instr::F32Trunc => {
                            let x = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(Val::Num(Num::F64(x.trunc())));
                            None
                        },
                        Instr::F32Nearest => {
                            let x = self.valueStack.pop().unwrap().to_f64() % 2.0;
                            
                            let ret = if x == 0.5 {
                                x.floor()
                            }else if x ==  -0.5{
                                x.ceil()
                            }else{
                                x.round()
                            };
                            self.valueStack.push(Val::Num(Num::F64(ret)));
                            None
                        },
                        Instr::LocalGet(idx) => {
                            self.valueStack.push(frame.locals[idx.0 as usize].clone());
                            None
                        },
                        Instr::LocalSet(idx) => {
                            frame.locals[idx.0 as usize] = self.valueStack.pop().unwrap();
                            None
                        },
                        Instr::I32Add => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a + b))
                            );
                            None
                        },
                        Instr::I32Sub => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a - b))
                            );
                            None
                        },
                        Instr::I32Mul => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a * b))
                            );
                            None
                        },
                        Instr::I32DivS => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a.checked_div(b).ok_or_else(|| RuntimeError::ZeroDivideError)?))
                            );
                            None
                        },
                        Instr::I32DivU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u32;
                            let b = self.valueStack.pop().unwrap().to_i32() as u32;
                            self.valueStack.push(
                                Val::Num(Num::I32(a.checked_div(b).ok_or_else(|| RuntimeError::ZeroDivideError)? as i32))
                            );
                            None
                        },
                        Instr::I32RemS => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a.overflowing_rem(b).0))
                            );
                            None
                        },
                        Instr::I32RemU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u32;
                            let b = self.valueStack.pop().unwrap().to_i32() as u32;
                            self.valueStack.push(
                                Val::Num(Num::I32(a.overflowing_rem(b).0 as i32))
                            );
                            None
                        },
                        Instr::I32And => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a & b))
                            );
                            None
                        },
                        Instr::I32Or => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a | b))
                            );
                            None
                        },
                        Instr::I32Xor => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a ^ b))
                            );
                            None
                        },
                        Instr::I32Shl => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a << b))
                            );
                            None
                        },
                        Instr::I32Shl => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a << b))
                            );
                            None
                        },
                        Instr::I32ShrS => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a >> b))
                            );
                            None
                        },
                        Instr::I32ShrU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u32;
                            let b = self.valueStack.pop().unwrap().to_i32() as u32;
                            self.valueStack.push(
                                Val::Num(Num::I32((a >> b) as i32))
                            );
                            None
                        },
                        Instr::I32Rotl => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a.rotate_left(b as u32)))
                            );
                            None
                        },
                        Instr::I32Rotr => {
                            let a = self.valueStack.pop().unwrap().to_i32();
                            let b = self.valueStack.pop().unwrap().to_i32();
                            self.valueStack.push(
                                Val::Num(Num::I32(a.rotate_right(b as u32)))
                            );
                            None
                        },
                        Instr::I64Add => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a + b))
                            );
                            None
                        },
                        Instr::I64Sub => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a - b))
                            );
                            None
                        },
                        Instr::I64Mul => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a * b))
                            );
                            None
                        },
                        Instr::I64DivS => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a.checked_div(b).ok_or_else(|| RuntimeError::ZeroDivideError)?))
                            );
                            None
                        },
                        Instr::I64DivU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u64;
                            let b = self.valueStack.pop().unwrap().to_i32() as u64;
                            self.valueStack.push(
                                Val::Num(Num::I64(a.checked_div(b).ok_or_else(|| RuntimeError::ZeroDivideError)? as i64))
                            );
                            None
                        },
                        Instr::I64RemS => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a.overflowing_rem(b).0))
                            );
                            None
                        },
                        Instr::I64RemU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u64;
                            let b = self.valueStack.pop().unwrap().to_i32() as u64;
                            self.valueStack.push(
                                Val::Num(Num::I64(a.overflowing_rem(b).0 as i64))
                            );
                            None
                        },
                        Instr::I64And => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a & b))
                            );
                            None
                        },
                        Instr::I64Or => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a | b))
                            );
                            None
                        },
                        Instr::I64Xor => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a ^ b))
                            );
                            None
                        },

                        Instr::I64Shl => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a << b))
                            );
                            None
                        },
                        Instr::I64Shl => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a << b))
                            );
                            None
                        },
                        Instr::I64ShrS => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a >> b))
                            );
                            None
                        },
                        Instr::I64ShrU => {
                            let a = self.valueStack.pop().unwrap().to_i32() as u64;
                            let b = self.valueStack.pop().unwrap().to_i32() as u64;
                            self.valueStack.push(
                                Val::Num(Num::I64((a >> b) as i64))
                            );
                            None
                        },
                        Instr::I64Rotl => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a.rotate_left(b as u32)))
                            );
                            None
                        },
                        Instr::I64Rotr => {
                            let a = self.valueStack.pop().unwrap().to_i64();
                            let b = self.valueStack.pop().unwrap().to_i64();
                            self.valueStack.push(
                                Val::Num(Num::I64(a.rotate_right(b as u32)))
                            );
                            None
                        },

                        Instr::F32Add => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a + b))
                            );
                            None
                        },
                        Instr::F32Sub => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a - b))
                            );
                            None
                        },
                        Instr::F32Mul => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a * b))
                            );
                            None
                        },
                        Instr::F32Div => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a / b))
                            );
                            None
                        },
                        Instr::F32Min => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a.min(b)))
                            );
                            None
                        },
                        Instr::F32Max => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a.max(b)))
                            );
                            None
                        },
                        Instr::F32Copysign => {
                            let a = self.valueStack.pop().unwrap().to_f32();
                            let b = self.valueStack.pop().unwrap().to_f32();
                            self.valueStack.push(
                                Val::Num(Num::F32(a.copysign(b)))
                            );
                            None
                        },
                        Instr::F64Add => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a + b))
                            );
                            None
                        },
                        Instr::F64Sub => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a - b))
                            );
                            None
                        },
                        Instr::F64Mul => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a * b))
                            );
                            None
                        },
                        Instr::F64Div => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a / b))
                            );
                            None
                        },
                        Instr::F64Min => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a.min(b)))
                            );
                            None
                        },
                        Instr::F64Max => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a.max(b)))
                            );
                            None
                        },
                        Instr::F64Copysign => {
                            let a = self.valueStack.pop().unwrap().to_f64();
                            let b = self.valueStack.pop().unwrap().to_f64();
                            self.valueStack.push(
                                Val::Num(Num::F64(a.copysign(b)))
                            );
                            None
                        },
                        _ => todo!(),
                    }
                },
                AdminInstr::FrameInstr(frame) => {
                    match frame{
                        FrameInstr::Br(idx) => Some(FrameInstr::Br(idx)),
                        FrameInstr::Label(label, instrs) => Some(FrameInstr::Label(label, instrs)),
                        FrameInstr::EndLabel => Some(FrameInstr::EndLabel),
                        FrameInstr::ModuleInstr(m) => Some(FrameInstr::ModuleInstr(m)),
                    }
                },
                AdminInstr::ModuleInstr(module) => {
                    match module{
                        ModuleInstr::Invoke(funcaddr) => Some(FrameInstr::ModuleInstr(ModuleInstr::Invoke(funcaddr))),
                        ModuleInstr::Return => Some(FrameInstr::ModuleInstr(ModuleInstr::Return)),
                    }
                }
                _ => todo!(),
            }
        } else{
            Some(FrameInstr::EndLabel)
        })
    }
}

#[derive(Clone)]
pub enum ModuleInstr{
    Invoke(FuncAddr),
    Return,
}
#[derive(Clone)]
pub enum FrameInstr{
    Br(LabelIdx),
    Label(Label, Vec<Instr>),
    EndLabel,
    ModuleInstr(ModuleInstr)
}

#[derive(Clone)]
pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Ref(FuncAddr),
    ModuleInstr(ModuleInstr),
    FrameInstr(FrameInstr),
    RefExtern(ExternAddr),
}