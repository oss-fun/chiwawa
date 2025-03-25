use super::{value::*, module::*, func::*};
use crate::structure::{instructions::Instr, types::*};
use crate::error::RuntimeError;
use std::rc::Weak;
use num::NumCast;
use std::cmp::{max, min};
use std::io::Cursor;
use byteorder::*;
use std::arch::asm;

pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Stacks{
        Stacks{
            activation_frame_stack: vec![
                FrameStack{
                    frame: Frame{
                        locals: Vec::new(),
                        module: Weak::new(),
                        n: 0,
                    },
                    label_stack: vec![
                        LabelStack{
                            label: Label{
                                continue_: vec![],
                                locals_num: 0,
                            },
                            instrs: vec![
                                AdminInstr::Invoke(funcaddr.clone())
                            ],
                            value_stack: params.clone(),
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
        let cur_frame = self.activation_frame_stack.last_mut().unwrap();
        if let Some(instr) = cur_frame.exec_instr_frame_level()? {
            let cur_label = cur_frame.label_stack.last_mut().unwrap();
            match instr {
                ModuleLevelInstr::Invoke(func_addr) => {
                    match &*func_addr.borrow(){
                        FuncInst::RuntimeFunc{type_,module,code} => {
                            let frame = FrameStack{
                                frame: Frame{
                                    locals: {
                                        let mut locals = Vec::new();
                                        locals.append(
                                            &mut cur_label.value_stack.split_off(cur_label.value_stack.len() - type_.params().len())
                                        );
                                        for v in code.locals.iter(){
                                            for _ in 0..(v.0){
                                                locals.push(
                                                    match v.1{
                                                        ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0 as i32)),
                                                        ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0 as i64)),
                                                        ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0 as f32)),
                                                        ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0 as f64)),
                                                        ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0 as i128)),
                                                        ValueType::RefType(_) => todo!(),
                                                    }
                                                );
                                            }
                                        };
                                        locals
                                    },
                                    module: module.clone(),
                                    n: type_.results.first().iter().count()
                                },
                                label_stack: vec![
                                    LabelStack{
                                        label: Label{
                                            continue_: vec![],
                                            locals_num: type_.results.iter().count(),
                                        },
                                        instrs: code.body.0.clone().into_iter().map(AdminInstr::Instr).rev().collect(),
                                        value_stack: vec![],
                                    }
                                ],
                                void:type_.results.iter().count() ==0 ,                           
                            };
                            self.activation_frame_stack.push(frame);

                        },
                        FuncInst::HostFunc{..} => {
                            todo!()
                        },
                    }

                },
                ModuleLevelInstr::Return =>{
                    let ret = cur_label.value_stack.pop();
                    let n = self.activation_frame_stack.pop().unwrap().frame.n;
                    if n != 0{
                        self.activation_frame_stack.last_mut().unwrap().label_stack.last_mut().unwrap().value_stack.push(ret.unwrap());
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
    pub n: usize,
}

pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
}

impl FrameStack{
    pub fn exec_instr_frame_level(&mut self) -> Result<Option<ModuleLevelInstr>, RuntimeError>{
        let cur_label = self.label_stack.last_mut().unwrap();
        if let Some(instr) = cur_label.exec_primitive_instr(&mut self.frame)?{
            match instr {
                /*Redirect to Exec_instr(Handing Instruction Spanning Frame)*/
                FrameLevelInstr::Invoke(idx) => Ok(Some(ModuleLevelInstr::Invoke(idx))),
                FrameLevelInstr::Return => Ok(Some(ModuleLevelInstr::Return)),
                FrameLevelInstr::Br(idx) => {
                    let idx = idx.to_usize();
                    let mut cur_label_value = self.label_stack.last().unwrap().value_stack.clone();
                    for _ in 0..idx{
                        self.label_stack.pop().unwrap();
                    };
                    
                    let continue_label = self.label_stack.pop().unwrap().label;
                    let mut instrs = continue_label.continue_.clone().into_iter().map(AdminInstr::Instr).rev().collect::<Vec<_>>();

                    if let Some(dst_label) = self.label_stack.last_mut(){
                        dst_label.instrs.append(&mut instrs);
                        let mut push = cur_label_value.split_off(cur_label_value.len()- continue_label.locals_num);
                        dst_label.value_stack.append(&mut push);
                        Ok(None)
                    }else{
                        self.label_stack.push(
                            LabelStack{
                                label: Label{
                                    continue_: vec![],
                                    locals_num: 0,
                                },
                                instrs: vec![],
                                value_stack: cur_label_value,
                            }
                        );
                        Ok(Some(ModuleLevelInstr::Return))
                    }
                },
                FrameLevelInstr::Label(label, instrs) => {
                    self.label_stack.push(
                        LabelStack{
                            label,
                            instrs: instrs.into_iter().map(AdminInstr::Instr).rev().collect(),
                            value_stack: vec![],
                        }
                    );
                    Ok(None)
                },
                FrameLevelInstr::EndLabel => {
                    let mut cur_label = self.label_stack.pop().unwrap();
                    if let Some(last) = self.label_stack.last_mut() {
                        last.value_stack.append(&mut cur_label.value_stack);
                        Ok(None)
                    } else {
                        self.label_stack.push(cur_label);
                        Ok(Some(ModuleLevelInstr::Return)) 
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
    pub value_stack: Vec<Val>,
}

impl LabelStack{
    pub fn exec_primitive_instr(&mut self, frame: &mut Frame) -> Result<Option<FrameLevelInstr>, RuntimeError>{
        Ok(if let Some(instr) = self.instrs.pop(){
            match instr {
                AdminInstr::Instr(instr) => {
                    match instr {
                        /*Single Operand Numeric Instructions*/
                        Instr::I32Const(x) => {
                            self.value_stack.push(Val::Num(Num::I32(x)));
                            None
                        },
                        Instr::I64Const(x) => {
                            self.value_stack.push(Val::Num(Num::I64(x)));
                            None
                        },
                        Instr::F32Const(x) => {
                            self.value_stack.push(Val::Num(Num::F32(x as f32)));
                            None
                        },
                        Instr::F64Const(x) => {
                            self.value_stack.push(Val::Num(Num::F64(x as f64)));
                            None
                        },
                        Instr::I32Clz => {
                            let x = self.value_stack.pop().unwrap().to_i32().leading_zeros();
                            self.value_stack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I32Ctz => {
                            let x = self.value_stack.pop().unwrap().to_i32().trailing_zeros();
                            self.value_stack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I32Popcnt => {
                            let x = self.value_stack.pop().unwrap().to_i32().count_ones();
                            self.value_stack.push(Val::Num(Num::I32(x as i32)));
                            None
                        },
                        Instr::I64Clz => {
                            let x = self.value_stack.pop().unwrap().to_i64().leading_zeros();
                            self.value_stack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::I64Ctz => {
                            let x = self.value_stack.pop().unwrap().to_i64().trailing_zeros();
                            self.value_stack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::I64Popcnt => {
                            let x = self.value_stack.pop().unwrap().to_i64().count_ones();
                            self.value_stack.push(Val::Num(Num::I64(x as i64)));
                            None
                        },
                        Instr::F32Abs => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x.abs())));
                            None
                        },
                        Instr::F32Neg => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x * -1.0)));
                            None
                        },
                        Instr::F32Sqrt => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x.sqrt())));
                            None
                        },
                        Instr::F32Ceil => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x.ceil())));
                            None
                        },
                        Instr::F32Floor => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x.floor())));
                            None
                        },
                        Instr::F32Trunc => {
                            let x = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(Val::Num(Num::F32(x.trunc())));
                            None
                        },
                        Instr::F32Nearest => {
                            let x = self.value_stack.pop().unwrap().to_f32() % 2.0;
                            
                            let ret = if x == 0.5 {
                                x.floor()
                            }else if x ==  -0.5{
                                x.ceil()
                            }else{
                                x.round()
                            };
                            self.value_stack.push(Val::Num(Num::F32(ret)));
                            None
                        },
                        Instr::F64Abs => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x.abs())));
                            None
                        },
                        Instr::F64Neg => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x * -1.0)));
                            None
                        },
                        Instr::F64Sqrt => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x.sqrt())));
                            None
                        },
                        Instr::F64Ceil => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x.ceil())));
                            None
                        },
                        Instr::F64Floor => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x.floor())));
                            None
                        },
                        Instr::F64Trunc => {
                            let x = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(Val::Num(Num::F64(x.trunc())));
                            None
                        },
                        Instr::F64Nearest => {
                            let x = self.value_stack.pop().unwrap().to_f64() % 2.0;
                            
                            let ret = if x == 0.5 {
                                x.floor()
                            }else if x ==  -0.5{
                                x.ceil()
                            }else{
                                x.round()
                            };
                            self.value_stack.push(Val::Num(Num::F64(ret)));
                            None
                        },
                        /*Two Operand Numeric Instructions*/
                        Instr::I32Add => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs+rhs))
                            );
                            None
                        },
                        Instr::I32Sub => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs - rhs))
                            );
                            None
                        },
                        Instr::I32Mul => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(rhs * lhs))
                            );
                            None
                        },
                        Instr::I32DivS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.checked_div(rhs).ok_or_else(|| RuntimeError::ZeroDivideError)?))
                            );
                            None
                        },
                        Instr::I32DivU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.checked_div(rhs).ok_or_else(|| RuntimeError::ZeroDivideError)? as i32))
                            );
                            None
                        },
                        Instr::I32RemS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.overflowing_rem(rhs).0))
                            );
                            None
                        },
                        Instr::I32RemU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.overflowing_rem(rhs).0 as i32))
                            );
                            None
                        },
                        Instr::I32And => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs & rhs))
                            );
                            None
                        },
                        Instr::I32Or => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs | rhs))
                            );
                            None
                        },
                        Instr::I32Xor => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs ^ rhs))
                            );
                            None
                        },
                        Instr::I32Shl => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs << rhs))
                            );
                            None
                        },
                        Instr::I32ShrS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs >> rhs))
                            );
                            None
                        },
                        Instr::I32ShrU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >> rhs) as i32))
                            );
                            None
                        },
                        Instr::I32Rotl => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.rotate_left(rhs as u32)))
                            );
                            None
                        },
                        Instr::I32Rotr => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32(lhs.rotate_right(rhs as u32)))
                            );
                            None
                        },
                        Instr::I64Add => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs + rhs))
                            );
                            None
                        },
                        Instr::I64Sub => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs - rhs))
                            );
                            None
                        },
                        Instr::I64Mul => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs * rhs))
                            );
                            None
                        },
                        Instr::I64DivS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.checked_div(rhs).ok_or_else(|| RuntimeError::ZeroDivideError)?))
                            );
                            None
                        },
                        Instr::I64DivU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.checked_div(rhs).ok_or_else(|| RuntimeError::ZeroDivideError)? as i64))
                            );
                            None
                        },
                        Instr::I64RemS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.overflowing_rem(rhs).0))
                            );
                            None
                        },
                        Instr::I64RemU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.overflowing_rem(rhs).0 as i64))
                            );
                            None
                        },
                        Instr::I64And => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs & rhs))
                            );
                            None
                        },
                        Instr::I64Or => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs | rhs))
                            );
                            None
                        },
                        Instr::I64Xor => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs ^ rhs))
                            );
                            None
                        },
                        Instr::I64Shl => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs << rhs))
                            );
                            None
                        },
                        Instr::I64ShrS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs >> rhs))
                            );
                            None
                        },
                        Instr::I64ShrU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I64((lhs >> rhs) as i64))
                            );
                            None
                        },
                        Instr::I64Rotl => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.rotate_left(rhs as u32)))
                            );
                            None
                        },
                        Instr::I64Rotr => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I64(lhs.rotate_right(rhs as u32)))
                            );
                            None
                        },

                        Instr::F32Add => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs + rhs))
                            );
                            None
                        },
                        Instr::F32Sub => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs - rhs))
                            );
                            None
                        },
                        Instr::F32Mul => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs * rhs))
                            );
                            None
                        },
                        Instr::F32Div => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs / rhs))
                            );
                            None
                        },
                        Instr::F32Min => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs.min(rhs)))
                            );
                            None
                        },
                        Instr::F32Max => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs.max(rhs)))
                            );
                            None
                        },
                        Instr::F32Copysign => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F32(lhs.copysign(rhs)))
                            );
                            None
                        },
                        Instr::F64Add => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs + rhs))
                            );
                            None
                        },
                        Instr::F64Sub => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs - rhs))
                            );
                            None
                        },
                        Instr::F64Mul => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs * rhs))
                            );
                            None
                        },
                        Instr::F64Div => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs / rhs))
                            );
                            None
                        },
                        Instr::F64Min => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs.min(rhs)))
                            );
                            None
                        },
                        Instr::F64Max => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs.max(rhs)))
                            );
                            None
                        },
                        Instr::F64Copysign => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F64(lhs.copysign(rhs)))
                            );
                            None
                        },
                        /*Numeric Comparison Instruction*/
                        Instr::I32Eqz => {
                            let a = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((a == 0) as i32))
                            );
                            None
                        },
                        Instr::I64Eqz => {
                            let a = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((a == 0) as i32))
                            );
                            None
                        },
                        Instr::I32Eq => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs == rhs) as i32))
                            );
                            None
                        },
                        Instr::I32Ne => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs != rhs) as i32))
                            );
                            None
                        },
                        Instr::I32LtS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::I32LtU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32()as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::I32GtS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::I32GtU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32()as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::I32LeS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::I32LeU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32()as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::I32GeS => {
                            let rhs = self.value_stack.pop().unwrap().to_i32();
                            let lhs = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },
                        Instr::I32GeU => {
                            let rhs = self.value_stack.pop().unwrap().to_i32()as u32;
                            let lhs = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },
                        Instr::I64Eq => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs == rhs) as i32))
                            );
                            None
                        },
                        Instr::I64Ne => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs != rhs) as i32))
                            );
                            None
                        },
                        Instr::I64LtS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::I64LtU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64()as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::I64GtS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::I64GtU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64()as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::I64LeS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::I64LeU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64()as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::I64GeS => {
                            let rhs = self.value_stack.pop().unwrap().to_i64();
                            let lhs = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },
                        Instr::I64GeU => {
                            let rhs = self.value_stack.pop().unwrap().to_i64()as u64;
                            let lhs = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Eq => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs == rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Ne => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs != rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Lt => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Gt => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Le => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::F32Ge => {
                            let rhs = self.value_stack.pop().unwrap().to_f32();
                            let lhs = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },

                        Instr::F64Eq => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs == rhs) as i32))
                            );
                            None
                        },
                        Instr::F64Ne => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs != rhs) as i32))
                            );
                            None
                        },
                        Instr::F64Lt => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs < rhs) as i32))
                            );
                            None
                        },
                        Instr::F64Gt => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs > rhs) as i32))
                            );
                            None
                        },
                        Instr::F64Le => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs <= rhs) as i32))
                            );
                            None
                        },
                        Instr::F64Ge => {
                            let rhs = self.value_stack.pop().unwrap().to_f64();
                            let lhs = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::I32((lhs >= rhs) as i32))
                            );
                            None
                        },
                        /*Type Translation Instructions*/
                        Instr::I32WrapI64 => {
                            let a = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::I32(a as i32))
                            );
                            None
                        },
                        Instr::I64ExtendI32S => {
                            let a = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::I64(a as i64))
                            );
                            None
                        },
                        Instr::I64ExtendI32U => {
                            let a = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::I64(a.into()))
                            );
                            None
                        },
                        Instr::I32Extend8S =>{
                            let a = self.value_stack.pop().unwrap().to_i32() as i8;
                            self.value_stack.push(
                                Val::Num(Num::I32(a.into()))
                            );
                            None
                        },
                        Instr::I32Extend16S =>{
                            let a = self.value_stack.pop().unwrap().to_i32() as i16;
                            self.value_stack.push(
                                Val::Num(Num::I32(a.into()))
                            );
                            None
                        },
                        Instr::I64Extend8S =>{
                            let a = self.value_stack.pop().unwrap().to_i64() as i8;
                            self.value_stack.push(
                                Val::Num(Num::I64(a.into()))
                            );
                            None
                        },
                        Instr::I64Extend16S =>{
                            let a = self.value_stack.pop().unwrap().to_i64() as i16;
                            self.value_stack.push(
                                Val::Num(Num::I64(a.into()))
                            );
                            None
                        },
                        Instr::I64Extend32S =>{
                            let a = self.value_stack.pop().unwrap().to_i64() as i32;
                            self.value_stack.push(
                                Val::Num(Num::I64(a.into()))
                            );
                            None
                        },
                        Instr::I32TruncF32S => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = <i32 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I32(result))
                            );
                            None
                        },
                        Instr::I32TruncF32U => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = <u32 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I32(result as i32))
                            );
                            None
                        },
                        Instr::I32TruncF64S => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = <i32 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I32(result))
                            );
                            None
                        },
                        Instr::I32TruncF64U => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = <u32 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I32(result as i32))
                            );
                            None
                        },
                        Instr::I64TruncF32S => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = <i64 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I64(result))
                            );
                            None
                        },
                        Instr::I64TruncF32U => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = <u64 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I64(result as i64))
                            );
                            None
                        },
                        Instr::I64TruncF64S => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = <i64 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I64(result))
                            );
                            None
                        },
                        Instr::I64TruncF64U => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = <u64 as NumCast>::from(a).ok_or_else(|| RuntimeError::TruncError).unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I64(result as i64))
                            );
                            None
                        },
                        Instr::I32TruncSatF32S => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() && a.is_sign_negative() {
                                i32::MIN
                            } else if a.is_infinite() {
                                i32::MAX
                            } else {
                                let truncated = a.trunc();
                                if truncated < i32::MIN as f32 {
                                    i32::MIN
                                } else if truncated > i32::MAX as f32 {
                                    i32::MAX
                                } else {
                                    truncated as i32
                                }
                            };
                            self.value_stack.push(Val::Num(Num::I32(result)));                            
                            None
                        },
                        Instr::I32TruncSatF32U => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_sign_negative() {
                                0
                            } else if a.is_infinite() {
                                u32::MAX
                            } else {
                                let truncated = a.trunc();
                                if truncated > u32::MAX as f32 {
                                    u32::MAX
                                } else {
                                    truncated as u32
                                }
                            } as i32;                        
                            self.value_stack.push(Val::Num(Num::I32(result)));
                            None
                        },
                        Instr::I32TruncSatF64S => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() && a.is_sign_negative() {
                                i32::MIN
                            } else if a.is_infinite() {
                                i32::MAX
                            } else {
                                let truncated = a.trunc();
                                if truncated < i32::MIN as f64 {
                                    i32::MIN
                                } else if truncated > i32::MAX as f64 {
                                    i32::MAX
                                } else {
                                    truncated as i32
                                }
                            };
                            self.value_stack.push(Val::Num(Num::I32(result)));
                            None
                        },
                        Instr::I32TruncSatF64U => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() {
                                if a.is_sign_negative() {
                                    0 
                                } else {
                                    u32::MAX
                                }
                            } else if a < 0.0 {
                                0 
                            } else if a >= 4294967296.0 {
                                u32::MAX
                            } else {
                                a.trunc() as u32
                            };
                            self.value_stack.push(Val::Num(Num::I32(result as i32)));
                            None
                        },
                        Instr::I64TruncSatF32S => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() {
                                if a.is_sign_negative() {
                                    i64::MIN
                                } else {
                                    i64::MAX
                                }
                            } else if a < i64::MIN as f32 {
                                i64::MIN
                            } else if a > i64::MAX as f32 {
                                i64::MAX
                            } else {
                                a.trunc() as i64
                            };        
                            self.value_stack.push(Val::Num(Num::I64(result)));
                            None
                        },
                        Instr::I64TruncSatF32U => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() {
                                if a.is_sign_negative() {
                                    0
                                } else {
                                    u64::MAX as i64
                                }
                            } else if a < 0.0 {
                                0 
                            } else if a > u64::MAX as f32 {
                                u64::MAX as i64
                            } else {
                                a.trunc() as u64 as i64
                            };
                            self.value_stack.push(Val::Num(Num::I64(result)));
                            None
                        },
                        Instr::I64TruncSatF64S => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = if a.is_nan() {
                                0
                            } else if a.is_infinite() {
                                if a.is_sign_negative() {
                                    i64::MIN
                                } else {
                                    i64::MAX
                                }
                            } else if a < i64::MIN as f64 {
                                i64::MIN
                            } else if a > i64::MAX as f64 {
                                i64::MAX
                            } else {
                                a.trunc() as i64
                            };
                            self.value_stack.push(Val::Num(Num::I64(result)));
                            None
                        },
                        Instr::I64TruncSatF64U => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let result = if a.is_nan() || a < 0.0 {
                                0
                            } else if a.is_infinite() {
                                u64::MAX as i64
                            } else if a > u64::MAX as f64 {
                                u64::MAX as i64
                            } else {
                                a.trunc() as u64 as i64
                            };
                            self.value_stack.push(Val::Num(Num::I64(result)));
                            None
                        },
                        Instr::F32DemoteF64 => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            self.value_stack.push(
                                Val::Num(Num::F32(a as f32))
                            );
                            None
                        },
                        Instr::F64PromoteF32 => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            self.value_stack.push(
                                Val::Num(Num::F64(a as f64))
                            );
                            None
                        },

                        Instr::F32ConvertI32S => {
                            let a = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::F32(a as f32))
                            );
                            None
                        },
                        Instr::F32ConvertI32U => {
                            let a = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::F32(a as f32))
                            );
                            None
                        },
                        Instr::F32ConvertI64S => {
                            let a = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::F32(a as f32))
                            );
                            None
                        },
                        Instr::F32ConvertI64U => {
                            let a = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::F32(a as f32))
                            );
                            None
                        },
                        Instr::F64ConvertI32S => {
                            let a = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(
                                Val::Num(Num::F64(a as f64))
                            );
                            None
                        },
                        Instr::F64ConvertI32U => {
                            let a = self.value_stack.pop().unwrap().to_i32() as u32;
                            self.value_stack.push(
                                Val::Num(Num::F64(a as f64))
                            );
                            None
                        },
                        Instr::F64ConvertI64S => {
                            let a = self.value_stack.pop().unwrap().to_i64();
                            self.value_stack.push(
                                Val::Num(Num::F64(a as f64))
                            );
                            None
                        },
                        Instr::F64ConvertI64U => {
                            let a = self.value_stack.pop().unwrap().to_i64() as u64;
                            self.value_stack.push(
                                Val::Num(Num::F64(a as f64))
                            );
                            None
                        },
                        Instr::I32ReinterpretF32 => {
                            let a = self.value_stack.pop().unwrap().to_f32();
                            let mut buf = vec![];
                            buf.write_f32::<LittleEndian>(a).unwrap();
                            let mut cursor = Cursor::new(buf);
                            let result = cursor.read_i32::<LittleEndian>().unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I32(result))
                            );
                            None
                        },
                        Instr::I64ReinterpretF64 => {
                            let a = self.value_stack.pop().unwrap().to_f64();
                            let mut buf = vec![];
                            buf.write_f64::<LittleEndian>(a).unwrap();
                            let mut cursor = Cursor::new(buf);
                            let result = cursor.read_i64::<LittleEndian>().unwrap();
                            self.value_stack.push(
                                Val::Num(Num::I64(result))
                            );
                            None
                        },
                        Instr::F32ReinterpretI32 => {
                            let a = self.value_stack.pop().unwrap().to_i32();
                            let mut buf = vec![];
                            buf.write_i32::<LittleEndian>(a).unwrap();
                            let mut cursor = Cursor::new(buf);
                            let result = cursor.read_f32::<LittleEndian>().unwrap();
                            self.value_stack.push(
                                Val::Num(Num::F32(result))
                            );
                            None
                        },

                        Instr::F64ReinterpretI64 => {
                            let a = self.value_stack.pop().unwrap().to_i64();
                            let mut buf = vec![];
                            buf.write_i64::<LittleEndian>(a).unwrap();
                            let mut cursor = Cursor::new(buf);
                            let result = cursor.read_f64::<LittleEndian>().unwrap();
                            self.value_stack.push(
                                Val::Num(Num::F64(result))
                            );
                            None
                        },
                        /*Variable Instructions*/
                        Instr::LocalGet(idx) => {
                            self.value_stack.push(frame.locals[idx.0 as usize].clone());
                            None
                        },
                        Instr::LocalSet(idx) => {
                            frame.locals[idx.0 as usize] = self.value_stack.pop().unwrap();
                            None
                        },
                        Instr::LocalTee(idx) => {
                            frame.locals[idx.0 as usize] = self.value_stack.last().unwrap().clone();
                            None
                        },
                        Instr::GlobalGet(idx) =>{
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(module_inst.global_addrs.get_by_idx(idx).get());
                            None
                        },
                        Instr::GlobalSet(idx) =>{
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.global_addrs.get_by_idx(idx).set(self.value_stack.pop().unwrap())?;
                            None
                        },
                        Instr::I32Load(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].load::<i32>(&arg, ptr)?)));
                            None
                        },
                        Instr::I64Load(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<i64>(&arg, ptr)?)));
                            None
                        },
                        Instr::F32Load(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::F32(module_inst.mem_addrs[0].load::<f32>(&arg, ptr)?)));
                            None
                        },
                        Instr::F64Load(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::F64(module_inst.mem_addrs[0].load::<f64>(&arg, ptr)?)));
                            None
                        },
                        Instr::I32Store(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i32();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i32>(&arg, ptr, data)?;
                            None
                        },
                        Instr::I64Store(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i64();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i64>(&arg, ptr, data)?;
                            None
                        },
                        Instr::F32Store(arg) => {
                            let data = self.value_stack.pop().unwrap().to_f32();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<f32>(&arg, ptr, data)?;
                            None
                        },
                        Instr::F64Store(arg) => {
                            let data = self.value_stack.pop().unwrap().to_f64();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<f64>(&arg, ptr, data)?;
                            None
                        },
                        Instr::I32Load8S(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].load::<i8>(&arg, ptr)? as i32)));
                            None
                        },
                        Instr::I32Load8U(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].load::<u8>(&arg, ptr)? as i32)));
                            None
                        },
                        Instr::I64Load8S(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<i8>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I64Load8U(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<u8>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I32Load16S(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].load::<i16>(&arg, ptr)? as i32)));
                            None
                        },
                        Instr::I32Load16U(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].load::<u16>(&arg, ptr)? as i32)));
                            None
                        },
                        Instr::I64Load16S(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<i16>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I64Load16U(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<u16>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I64Load32S(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<i32>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I64Load32U(arg) =>{
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I64(module_inst.mem_addrs[0].load::<u32>(&arg, ptr)? as i64)));
                            None
                        },
                        Instr::I32Store8(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i32();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i8>(&arg, ptr, data as i8)?;
                            None
                        },
                        Instr::I64Store8(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i64();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i8>(&arg, ptr, data as i8)?;
                            None
                        },
                        Instr::I32Store16(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i32();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i16>(&arg, ptr, data as i16)?;
                            None
                        },
                        Instr::I64Store16(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i64();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i16>(&arg, ptr, data as i16)?;
                            None
                        },
                        Instr::I64Store32(arg) => {
                            let data = self.value_stack.pop().unwrap().to_i64();
                            let ptr = self.value_stack.pop().unwrap().to_i32();
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            module_inst.mem_addrs[0].store::<i32>(&arg, ptr, data as i32)?;
                            None
                        },
                        Instr::MemorySize =>{
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].mem_size())));

                            None
                        },
                        Instr::MemoryGrow => {
                            let module_inst = frame.module.upgrade().ok_or_else(||RuntimeError::InstructionFailed).unwrap();
                            let size = self.value_stack.pop().unwrap().to_i32();
                            self.value_stack.push(Val::Num(Num::I32(module_inst.mem_addrs[0].mem_grow(size))));
                            None
                        },
                        Instr::Drop => {
                            let _ = self.value_stack.pop();
                            None
                        },
                        Instr::Select(_) => {
                            let cond = self.value_stack.pop().unwrap().to_i32();
                            let v2 = self.value_stack.pop().unwrap();
                            let v1 = self.value_stack.pop().unwrap();
                            if cond == 0 {
                                self.value_stack.push(v2);
                            }else{
                                self.value_stack.push(v1);
                            }
                            None

                        },
                        Instr::V128Const(_) | Instr::V128Not | Instr::V128And | Instr::V128AndNot | Instr::V128Or | Instr::V128Xor | Instr::V128Bitselect | Instr::V128AnyTrue | 
                        Instr::I8x16Shuffle(_) | Instr::I8x16Swizzle | Instr::I8x16Splat | Instr::I16x8Splat | Instr::I32x4Splat | Instr::I64x2Splat | Instr::F32x4Splat | Instr::F64x2Splat |
                        Instr::I8x16ExtractLaneU(_) | Instr::I8x16ExtractLaneS(_) | Instr::I16x8ExtractLaneU(_) | Instr::I16x8ExtractLaneS(_) | Instr::I32x4ExtractLane(_) | Instr::I64x2ExtractLane(_) |
                        Instr::F32x4ExtractLane(_) | Instr::F64x2ExtractLane(_) | Instr::I16x8ReplaceLane(_)| Instr::I32x4ReplaceLane(_) | Instr::I8x16ReplaceLane(_) | Instr::I64x2ReplaceLane(_) | Instr::F32x4ReplaceLane(_) |
                        Instr::F64x2ReplaceLane(_) | Instr::I8x16Eq |  Instr::I8x16Ne | Instr::I8x16LtU | Instr::I8x16LtS| Instr::I8x16GtU| Instr::I8x16GtS| Instr::I8x16LeU|
                        Instr::I8x16LeS| Instr::I8x16GeU| Instr::I8x16GeS| Instr::I16x8Eq| Instr::I16x8Ne| Instr::I16x8LtU| Instr::I16x8LtS| Instr::I16x8GtU| Instr::I16x8GtS| Instr::I16x8LeU|
                        Instr::I16x8LeS| Instr::I16x8GeU| Instr::I16x8GeS| Instr::I32x4Eq| Instr::I32x4Ne| Instr::I32x4LtU| Instr::I32x4LtS| Instr::I32x4GtU| Instr::I32x4GtS| Instr::I32x4LeU| Instr::I32x4LeS| Instr::I32x4GeU|
                        Instr::I32x4GeS| Instr::I64x2Eq| Instr::I64x2Ne| Instr::I64x2LtS| Instr::I64x2GtS| Instr::I64x2LeS| Instr::I64x2GeS| Instr::F32x4Eq| Instr::F32x4Ne| Instr::F32x4Lt| Instr::F32x4Gt| Instr::F32x4Le| Instr::F32x4Ge|
                        Instr::F64x2Eq| Instr::F64x2Ne| Instr::F64x2Lt| Instr::F64x2Gt| Instr::F64x2Le| Instr::F64x2Ge| Instr::I8x16Abs| Instr::I16x8Abs| Instr::I32x4Abs| Instr::I64x2Abs| Instr::I8x16Neg| Instr::I16x8Neg| Instr::I32x4Neg|
                        Instr::I64x2Neg| Instr::I8x16PopCnt| Instr::I16x8Q15MulrSatS| Instr::I32x4DotI16x8S| Instr::F32x4Abs| Instr::F32x4Neg| Instr::F32x4Sqrt| Instr::F32x4Ceil| Instr::F32x4Floor| Instr::F32x4Truc|
                        Instr::F32x4Mearest| Instr::F64x2Abs| Instr::F64x2Neg| Instr::F64x2Sqrt| Instr::F64x2Ceil| Instr::F64x2Floor| Instr::F64x2Truc| Instr::F64x2Mearest| Instr::I8x16AllTrue| Instr::I16x8Alltrue|
                        Instr::I32x4Alltrue| Instr::I64x2Alltrue| Instr::I8x16Bitmask| Instr::I16x8Bitmask| Instr::I32x4Bitmask| Instr::I64x2Bitmask| Instr::I8x16NarrowI16x8U| Instr::I8x16NarrowI16x8S| Instr::I16x8NarrowI32x4U|
                        Instr::I16x8NarrowI32x4S| Instr::I16x8ExtendHalfI8x16U| Instr::I16x8ExtendHalfI8x16S| Instr::I32x4ExtendHalfI16x8U| Instr::I32x4ExtendHalfI16x8S| Instr::I64x2ExtendHalfI32x4U|Instr::I64x2ExtendHalfI32x4S|
                        Instr::I8x16Shl| Instr::I8x16ShrU| Instr::I8x16ShrS| Instr::I16x8Shl| Instr::I16x8ShrU| Instr::I16xShrS| Instr::I32x4Shl|Instr::I32x4ShrU| Instr::I32x4ShrS| Instr::I64x2Shl| Instr::I64x2ShrU| Instr::I64x2ShrS|
                        Instr::I8x16Add| Instr::I8x16Sub| Instr::I16x8Add| Instr::I16x8Sub| Instr::I32x4Add| Instr::I32x4Sub| Instr::I64x2Add| Instr::I64x2Sub| Instr::I8x16MinU| Instr::I8x16MinS| Instr::I8x16MaxU| Instr::I8x16MaxS|
                        Instr::I16x8MinU| Instr::I16x8MinS| Instr::I16x8MaxU| Instr::I16x8MaxS| Instr::I32x4MinU| Instr::I32x4MinS| Instr::I32x4MaxU| Instr::I32x4MaxS| Instr::I8x16AddSatU| Instr::I8x16AddSatS|
                        Instr::I8x16SubSatU| Instr::I8x16SubSatS| Instr::I16x8AddSatU| Instr::I16x8AddSatS| Instr::I16x8SubSatU| Instr::I16x8SubSatS| Instr::I16x8Mul| Instr::I32x4Mul| Instr::I64x2Mul| Instr::I8x16AvgrU|
                        Instr::I16x8AvgrU| Instr::I16x8ExtmulHalfI8x16U| Instr::I16x8ExtmulHalfI8x16S| Instr::I32x4ExtmulHalfI16x8U| Instr::I32x4ExtmulHalfI16x8S| Instr::I64x2ExtmulHalfI32x4U| Instr::I64x2ExtmulHalfI32x4S| Instr::I16x8ExtaddPairwiseI8x16U|
                        Instr::I16x8ExtaddPairwiseI8x16S| Instr::I32x4ExtaddPairwiseI16x8U| Instr::I32x4ExtaddPairwiseI16x8S| Instr::F32x4Add| Instr::F32x4Sub| Instr::F32x4Mul| Instr::F32x4Div| Instr::F32x4Min| Instr::F32x4Max| Instr::F32x4Pmin|
                        Instr::F32x4Pmax| Instr::F64x2Add| Instr::F64x2Sub| Instr::F64x2Mul| Instr::F64x2Div| Instr::F64x2Min| Instr::F64x2Max| Instr::F64x2Pmin| Instr::F64x2Pmax| Instr::I32x4TruncSatF32x4U| Instr::I32x4TruncSatF32x4S|
                        Instr::I32x4TruncSatF64x2UZero| Instr::I32x4TruncSatF64x2SZero| Instr::F32x4ConvertI32x4U| Instr::F32x4ConvertI32x4S| Instr::F32x4DemoteF64x2Zero| Instr::F64x2ConvertLowI32x4U| Instr::F64x2ConvertLowI32x4S| Instr::F64x2PromoteLowF32x4
                        =>{
                            todo!()
                        },
                        Instr::RefNull(_)| Instr::RefIsNull| Instr::RefFunc(_) => {
                            todo!()
                        },
                        Instr::TableGet(_)| Instr::TableSet(_)| Instr::TableSize(_)| Instr::TableGrow(_)| Instr::TableFill(_)| Instr::TableCopy(_,_)| Instr::TableInit(_,_)| Instr::ElemDrop(_) => {
                            todo!()
                        },
                        Instr::V128Load(_)| Instr::V128Store(_)| Instr::V128Load8x8S(_)| Instr::V128Load8x8U(_)| Instr::V128Load16x4S(_)| Instr::V128Load16x4U(_)| Instr::V128Load32x2S(_)| Instr::V128Load32x2U(_)| Instr::V128Load8Splat(_)|
                        Instr::V128Load16Splat(_)| Instr::V128Load32Splat(_)| Instr::V128Load64Splat(_)| Instr::V128Load8lane(_, _)| Instr::V128Load16lane(_, _)| Instr::V128Load32lane(_, _)| Instr::V128Load32Zero(_)| Instr::V128Load64Zero(_)|
                        Instr::V128Load64lane(_, _)| Instr::V128Store8lane(_, _)| Instr::V128Store16lane(_, _)| Instr::V128Store32lane(_, _)| Instr::V128Store64lane(_, _) => {
                            todo!()
                        },
                        Instr::MemoryFill=>{
                            todo!()
                        },
                        Instr::MemoryCopy => {
                            todo!()
                        },
                        Instr::MemoryInit(_) => {
                            todo!()
                        },
                        Instr::DataDrop(_) => {
                            todo!()
                        },
                        Instr::Nop => None,
                        Instr::Unreachable => return Err(RuntimeError::Unreachable),
                        Instr::Block(type_, instrs) => {
                            self.instrs.push(AdminInstr::Label(
                                Label{
                                    continue_: vec![],
                                    locals_num: type_.1.iter().count(),
                                },
                                instrs
                            ));
                            None
                        }
                        Instr::Loop(type_, instrs) => {
                            self.instrs.push(AdminInstr::Label(
                                Label{
                                    continue_: vec![Instr::Loop(type_.clone(), instrs.clone())],
                                    locals_num: 0,
                                },
                                instrs
                            ));
                            None
                        },
                        Instr::If(type_, i1, i2) =>{
                            let bool_ = self.value_stack.pop().unwrap().to_i32();
                            self.instrs.push(AdminInstr::Label(
                                Label{
                                    continue_: vec![],
                                    locals_num: type_.1.iter().count(),
                                },
                                if  bool_ != 0 {i1} else{i2},
                            ));
                            None
                        },
                        Instr::BrIf(idx) =>{
                            let bool_ = self.value_stack.pop().unwrap().to_i32();
                            if bool_ != 0 {
                                self.instrs.push(AdminInstr::Br(idx))
                            }
                            None
                        },
                        Instr::BrTable(idxes,idx) =>{
                            let i = self.value_stack.pop().unwrap().to_i32();
                            if (i as usize) < idxes.len(){
                                self.instrs.push(AdminInstr::Br(idxes[i as usize].clone()));
                            }else{
                                self.instrs.push(AdminInstr::Br(idx));
                            }
                            None
                        },
                        Instr::Call(idx) =>{
                            let instance = frame.module.upgrade().unwrap();
                            self.instrs.push(AdminInstr::Invoke(instance.func_addrs.get_by_idx(idx).clone()));
                            None
                        },
                        Instr::Br(idx) => {
                            self.instrs.push(AdminInstr::Br(idx));
                            None
                        },
                        Instr::Return => {
                            self.instrs.push(AdminInstr::Return);
                            None
                        },
                        Instr::CallIndirect(tableidx, typeidx) => {
                            let instance = frame.module.upgrade().unwrap();
                            let table = instance.table_addrs.get_by_idx(tableidx);
                            let i = self.value_stack.pop().unwrap().to_i32();

                            let func = {
                                if let Some(func) = table.get(i as usize) {
                                    func
                                } else {
                                    return Err(RuntimeError::ExecutionFailed);
                                }
                            };

                            if func.func_type() != *instance.types.get_by_idx(typeidx){
                                return Err(RuntimeError::ExecutionFailed);
                            }
                            
                            self.instrs.push(AdminInstr::Invoke(func.clone()));
                            None
                        },
                    }
                },
                AdminInstr::Invoke(idx) => Some(FrameLevelInstr::Invoke(idx)),
                AdminInstr::Return =>Some(FrameLevelInstr::Return),
                AdminInstr::Label(label, instrs) => Some(FrameLevelInstr::Label(label, instrs)),
                AdminInstr::Br(idx) => Some(FrameLevelInstr::Br(idx)),
                _ => todo!(),
            }
        } else{
            Some(FrameLevelInstr::EndLabel)
        })
    }
}

#[derive(Clone)]
pub enum FrameLevelInstr{
    Label(Label, Vec<Instr>),
    Br(LabelIdx),
    EndLabel,
    Invoke(FuncAddr),
    Return
}

#[derive(Clone)]
pub enum ModuleLevelInstr{
    Invoke(FuncAddr),
    Return,
}

#[derive(Clone)]
pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Br(LabelIdx),
    Return,
    Ref(FuncAddr),
    RefExtern(ExternAddr),
}