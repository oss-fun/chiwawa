use crate::execution::stack::*;
use crate::structure::{instructions::Memarg, types::LocalIdx};
use wasmparser;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstType {
    I32,
    I64,
    F32,
    F64,
}

pub trait ConstHandler {
    type ValueType: Copy;

    fn create_local_set_operand(local_idx: u32, value: Self::ValueType) -> Operand;
    fn create_value_operand(value: Self::ValueType) -> Operand;
    fn get_const_type() -> ConstType;
    fn get_local_set_handler_idx() -> usize;

    fn create_memarg_operand(value: Self::ValueType, memarg: Memarg) -> Option<Operand> {
        None // Default: not supported
    }

    fn supports_load_store() -> bool {
        false
    }

    fn supports_shift() -> bool {
        false
    }

    fn supports_rotation() -> bool {
        false
    }

    fn try_consume_arithmetic(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        None // Default: no arithmetic operations
    }

    fn try_consume_load(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        None // Default: no load operations
    }

    fn try_consume_comparison(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        None // Default: no comparison operations
    }

    fn try_consume_store(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        None // Default: no store operations
    }

    fn try_consume_shift(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        None // Default: no shift operations
    }

    fn try_consume_rotation(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        None // Default: no rotation operations
    }

    fn try_consume_conversion(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        None // Default: no conversion operations
    }
}

pub struct I32Handler;

impl ConstHandler for I32Handler {
    type ValueType = i32;

    fn create_local_set_operand(local_idx: u32, value: i32) -> Operand {
        Operand::LocalIdxI32(LocalIdx(local_idx), value)
    }

    fn create_value_operand(value: i32) -> Operand {
        Operand::I32(value)
    }

    fn get_const_type() -> ConstType {
        ConstType::I32
    }

    fn get_local_set_handler_idx() -> usize {
        HANDLER_IDX_LOCAL_SET_I32_CONST
    }

    fn create_memarg_operand(value: i32, memarg: Memarg) -> Option<Operand> {
        Some(Operand::MemArgI32(value, memarg))
    }

    fn supports_load_store() -> bool {
        true
    }

    fn supports_shift() -> bool {
        true
    }

    fn supports_rotation() -> bool {
        true
    }

    fn try_consume_arithmetic(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Add => Some(HANDLER_IDX_I32_ADD_CONST),
                wasmparser::Operator::I32Sub => Some(HANDLER_IDX_I32_SUB_CONST),
                wasmparser::Operator::I32Mul => Some(HANDLER_IDX_I32_MUL_CONST),
                wasmparser::Operator::I32DivS => Some(HANDLER_IDX_I32_DIV_S_CONST),
                wasmparser::Operator::I32DivU => Some(HANDLER_IDX_I32_DIV_U_CONST),
                wasmparser::Operator::I32RemS => Some(HANDLER_IDX_I32_REM_S_CONST),
                wasmparser::Operator::I32RemU => Some(HANDLER_IDX_I32_REM_U_CONST),
                wasmparser::Operator::I32And => Some(HANDLER_IDX_I32_AND_CONST),
                wasmparser::Operator::I32Or => Some(HANDLER_IDX_I32_OR_CONST),
                wasmparser::Operator::I32Xor => Some(HANDLER_IDX_I32_XOR_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_load(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Load { memarg } => Some((
                    HANDLER_IDX_I32_LOAD_I32_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Load8S { memarg } => Some((
                    HANDLER_IDX_I32_LOAD8_S_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Load8U { memarg } => Some((
                    HANDLER_IDX_I32_LOAD8_U_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Load16S { memarg } => Some((
                    HANDLER_IDX_I32_LOAD16_S_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Load16U { memarg } => Some((
                    HANDLER_IDX_I32_LOAD16_U_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_comparison(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Eq => Some(HANDLER_IDX_I32_EQ_CONST),
                wasmparser::Operator::I32Ne => Some(HANDLER_IDX_I32_NE_CONST),
                wasmparser::Operator::I32LtS => Some(HANDLER_IDX_I32_LT_S_CONST),
                wasmparser::Operator::I32LtU => Some(HANDLER_IDX_I32_LT_U_CONST),
                wasmparser::Operator::I32GtS => Some(HANDLER_IDX_I32_GT_S_CONST),
                wasmparser::Operator::I32GtU => Some(HANDLER_IDX_I32_GT_U_CONST),
                wasmparser::Operator::I32LeS => Some(HANDLER_IDX_I32_LE_S_CONST),
                wasmparser::Operator::I32LeU => Some(HANDLER_IDX_I32_LE_U_CONST),
                wasmparser::Operator::I32GeS => Some(HANDLER_IDX_I32_GE_S_CONST),
                wasmparser::Operator::I32GeU => Some(HANDLER_IDX_I32_GE_U_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_store(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Store { memarg } => Some((
                    HANDLER_IDX_I32_STORE_I32_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Store8 { memarg } => Some((
                    HANDLER_IDX_I32_STORE8_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I32Store16 { memarg } => Some((
                    HANDLER_IDX_I32_STORE16_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_shift(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Shl => Some(HANDLER_IDX_I32_SHL_CONST),
                wasmparser::Operator::I32ShrS => Some(HANDLER_IDX_I32_SHR_S_CONST),
                wasmparser::Operator::I32ShrU => Some(HANDLER_IDX_I32_SHR_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_rotation(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32Rotl => Some(HANDLER_IDX_I32_ROTL_CONST),
                wasmparser::Operator::I32Rotr => Some(HANDLER_IDX_I32_ROTR_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_conversion(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64ExtendI32S => Some(HANDLER_IDX_I64_EXTEND_I32_S_CONST),
                wasmparser::Operator::I64ExtendI32U => Some(HANDLER_IDX_I64_EXTEND_I32_U_CONST),
                wasmparser::Operator::F32ConvertI32S => Some(HANDLER_IDX_F32_CONVERT_I32_S_CONST),
                wasmparser::Operator::F32ConvertI32U => Some(HANDLER_IDX_F32_CONVERT_I32_U_CONST),
                wasmparser::Operator::F64ConvertI32S => Some(HANDLER_IDX_F64_CONVERT_I32_S_CONST),
                wasmparser::Operator::F64ConvertI32U => Some(HANDLER_IDX_F64_CONVERT_I32_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }
}

pub struct I64Handler;

impl ConstHandler for I64Handler {
    type ValueType = i64;

    fn create_local_set_operand(local_idx: u32, value: i64) -> Operand {
        Operand::LocalIdxI64(LocalIdx(local_idx), value)
    }

    fn create_value_operand(value: i64) -> Operand {
        Operand::I64(value)
    }

    fn get_const_type() -> ConstType {
        ConstType::I64
    }

    fn get_local_set_handler_idx() -> usize {
        HANDLER_IDX_LOCAL_SET_I64_CONST
    }

    fn create_memarg_operand(value: i64, memarg: Memarg) -> Option<Operand> {
        Some(Operand::MemArgI64(value, memarg))
    }

    fn supports_load_store() -> bool {
        true
    }

    fn supports_shift() -> bool {
        true
    }

    fn supports_rotation() -> bool {
        true
    }

    fn try_consume_arithmetic(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Add => Some(HANDLER_IDX_I64_ADD_CONST),
                wasmparser::Operator::I64Sub => Some(HANDLER_IDX_I64_SUB_CONST),
                wasmparser::Operator::I64Mul => Some(HANDLER_IDX_I64_MUL_CONST),
                wasmparser::Operator::I64DivS => Some(HANDLER_IDX_I64_DIV_S_CONST),
                wasmparser::Operator::I64DivU => Some(HANDLER_IDX_I64_DIV_U_CONST),
                wasmparser::Operator::I64RemS => Some(HANDLER_IDX_I64_REM_S_CONST),
                wasmparser::Operator::I64RemU => Some(HANDLER_IDX_I64_REM_U_CONST),
                wasmparser::Operator::I64And => Some(HANDLER_IDX_I64_AND_CONST),
                wasmparser::Operator::I64Or => Some(HANDLER_IDX_I64_OR_CONST),
                wasmparser::Operator::I64Xor => Some(HANDLER_IDX_I64_XOR_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_load(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Load { memarg } => Some((
                    HANDLER_IDX_I64_LOAD_I64_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load8S { memarg } => Some((
                    HANDLER_IDX_I64_LOAD8_S_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load8U { memarg } => Some((
                    HANDLER_IDX_I64_LOAD8_U_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load16S { memarg } => Some((
                    HANDLER_IDX_I64_LOAD16_S_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load16U { memarg } => Some((
                    HANDLER_IDX_I64_LOAD16_U_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load32S { memarg } => Some((
                    HANDLER_IDX_I64_LOAD32_S_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Load32U { memarg } => Some((
                    HANDLER_IDX_I64_LOAD32_U_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_comparison(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Eq => Some(HANDLER_IDX_I64_EQ_CONST),
                wasmparser::Operator::I64Ne => Some(HANDLER_IDX_I64_NE_CONST),
                wasmparser::Operator::I64LtS => Some(HANDLER_IDX_I64_LT_S_CONST),
                wasmparser::Operator::I64LtU => Some(HANDLER_IDX_I64_LT_U_CONST),
                wasmparser::Operator::I64GtS => Some(HANDLER_IDX_I64_GT_S_CONST),
                wasmparser::Operator::I64GtU => Some(HANDLER_IDX_I64_GT_U_CONST),
                wasmparser::Operator::I64LeS => Some(HANDLER_IDX_I64_LE_S_CONST),
                wasmparser::Operator::I64LeU => Some(HANDLER_IDX_I64_LE_U_CONST),
                wasmparser::Operator::I64GeS => Some(HANDLER_IDX_I64_GE_S_CONST),
                wasmparser::Operator::I64GeU => Some(HANDLER_IDX_I64_GE_U_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_store(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<(usize, Memarg)> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Store { memarg } => Some((
                    HANDLER_IDX_I64_STORE_I64_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Store8 { memarg } => Some((
                    HANDLER_IDX_I64_STORE8_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Store16 { memarg } => Some((
                    HANDLER_IDX_I64_STORE16_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                wasmparser::Operator::I64Store32 { memarg } => Some((
                    HANDLER_IDX_I64_STORE32_CONST,
                    Memarg {
                        offset: memarg.offset as u32,
                        align: memarg.align as u32,
                    },
                )),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_shift(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Shl => Some(HANDLER_IDX_I64_SHL_CONST),
                wasmparser::Operator::I64ShrS => Some(HANDLER_IDX_I64_SHR_S_CONST),
                wasmparser::Operator::I64ShrU => Some(HANDLER_IDX_I64_SHR_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_rotation(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I64Rotl => Some(HANDLER_IDX_I64_ROTL_CONST),
                wasmparser::Operator::I64Rotr => Some(HANDLER_IDX_I64_ROTR_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_conversion(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32WrapI64 => Some(HANDLER_IDX_I32_WRAP_I64_CONST),
                wasmparser::Operator::F32ConvertI64S => Some(HANDLER_IDX_F32_CONVERT_I64_S_CONST),
                wasmparser::Operator::F32ConvertI64U => Some(HANDLER_IDX_F32_CONVERT_I64_U_CONST),
                wasmparser::Operator::F64ConvertI64S => Some(HANDLER_IDX_F64_CONVERT_I64_S_CONST),
                wasmparser::Operator::F64ConvertI64U => Some(HANDLER_IDX_F64_CONVERT_I64_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }
}

pub struct F32Handler;

impl ConstHandler for F32Handler {
    type ValueType = f32;

    fn create_local_set_operand(local_idx: u32, value: f32) -> Operand {
        Operand::LocalIdxF32(LocalIdx(local_idx), value)
    }

    fn create_value_operand(value: f32) -> Operand {
        Operand::F32(value)
    }

    fn get_const_type() -> ConstType {
        ConstType::F32
    }

    fn get_local_set_handler_idx() -> usize {
        HANDLER_IDX_LOCAL_SET_F32_CONST
    }

    fn try_consume_arithmetic(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::F32Add => Some(HANDLER_IDX_F32_ADD_CONST),
                wasmparser::Operator::F32Sub => Some(HANDLER_IDX_F32_SUB_CONST),
                wasmparser::Operator::F32Mul => Some(HANDLER_IDX_F32_MUL_CONST),
                wasmparser::Operator::F32Div => Some(HANDLER_IDX_F32_DIV_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_comparison(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::F32Eq => Some(HANDLER_IDX_F32_EQ_CONST),
                wasmparser::Operator::F32Ne => Some(HANDLER_IDX_F32_NE_CONST),
                wasmparser::Operator::F32Lt => Some(HANDLER_IDX_F32_LT_CONST),
                wasmparser::Operator::F32Gt => Some(HANDLER_IDX_F32_GT_CONST),
                wasmparser::Operator::F32Le => Some(HANDLER_IDX_F32_LE_CONST),
                wasmparser::Operator::F32Ge => Some(HANDLER_IDX_F32_GE_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_conversion(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32TruncF32S => Some(HANDLER_IDX_I32_TRUNC_F32_S_CONST),
                wasmparser::Operator::I32TruncF32U => Some(HANDLER_IDX_I32_TRUNC_F32_U_CONST),
                wasmparser::Operator::I64TruncF32S => Some(HANDLER_IDX_I64_TRUNC_F32_S_CONST),
                wasmparser::Operator::I64TruncF32U => Some(HANDLER_IDX_I64_TRUNC_F32_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }
}

pub struct F64Handler;

impl ConstHandler for F64Handler {
    type ValueType = f64;

    fn create_local_set_operand(local_idx: u32, value: f64) -> Operand {
        Operand::LocalIdxF64(LocalIdx(local_idx), value)
    }

    fn create_value_operand(value: f64) -> Operand {
        Operand::F64(value)
    }

    fn get_const_type() -> ConstType {
        ConstType::F64
    }

    fn get_local_set_handler_idx() -> usize {
        HANDLER_IDX_LOCAL_SET_F64_CONST
    }

    fn try_consume_arithmetic(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::F64Add => Some(HANDLER_IDX_F64_ADD_CONST),
                wasmparser::Operator::F64Sub => Some(HANDLER_IDX_F64_SUB_CONST),
                wasmparser::Operator::F64Mul => Some(HANDLER_IDX_F64_MUL_CONST),
                wasmparser::Operator::F64Div => Some(HANDLER_IDX_F64_DIV_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_comparison(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::F64Eq => Some(HANDLER_IDX_F64_EQ_CONST),
                wasmparser::Operator::F64Ne => Some(HANDLER_IDX_F64_NE_CONST),
                wasmparser::Operator::F64Lt => Some(HANDLER_IDX_F64_LT_CONST),
                wasmparser::Operator::F64Gt => Some(HANDLER_IDX_F64_GT_CONST),
                wasmparser::Operator::F64Le => Some(HANDLER_IDX_F64_LE_CONST),
                wasmparser::Operator::F64Ge => Some(HANDLER_IDX_F64_GE_CONST),
                _ => None,
            };

            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }

    fn try_consume_conversion(
        ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
    ) -> Option<usize> {
        if let Some(Ok((next_op, _))) = ops.peek() {
            let result = match next_op {
                wasmparser::Operator::I32TruncF64S => Some(HANDLER_IDX_I32_TRUNC_F64_S_CONST),
                wasmparser::Operator::I32TruncF64U => Some(HANDLER_IDX_I32_TRUNC_F64_U_CONST),
                wasmparser::Operator::I64TruncF64S => Some(HANDLER_IDX_I64_TRUNC_F64_S_CONST),
                wasmparser::Operator::I64TruncF64U => Some(HANDLER_IDX_I64_TRUNC_F64_U_CONST),
                _ => None,
            };
            if result.is_some() {
                let _ = ops.next().unwrap().unwrap();
            }
            result
        } else {
            None
        }
    }
}

pub fn try_consume_local_set(
    ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
) -> Option<u32> {
    if let Some(Ok((next_op, _))) = ops.peek() {
        if let wasmparser::Operator::LocalSet { local_index } = next_op {
            let local_idx = *local_index;
            let _ = ops.next().unwrap().unwrap();
            Some(local_idx)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn handle_const_patterns<H: ConstHandler>(
    value: H::ValueType,
    ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
) -> Option<ProcessedInstr> {
    if let Some(local_idx) = try_consume_local_set(ops) {
        return Some(ProcessedInstr {
            handler_index: H::get_local_set_handler_idx(),
            operand: H::create_local_set_operand(local_idx, value),
        });
    }

    if H::supports_load_store() {
        if let Some((handler_index, memarg)) = H::try_consume_load(ops) {
            if let Some(operand) = H::create_memarg_operand(value, memarg) {
                return Some(ProcessedInstr {
                    handler_index,
                    operand,
                });
            }
        }
    }

    if H::supports_load_store() {
        if let Some((handler_index, memarg)) = H::try_consume_store(ops) {
            if let Some(operand) = H::create_memarg_operand(value, memarg) {
                return Some(ProcessedInstr {
                    handler_index,
                    operand,
                });
            }
        }
    }

    if let Some(handler_index) = H::try_consume_comparison(ops) {
        return Some(ProcessedInstr {
            handler_index,
            operand: H::create_value_operand(value),
        });
    }

    if H::supports_shift() {
        if let Some(handler_index) = H::try_consume_shift(ops) {
            return Some(ProcessedInstr {
                handler_index,
                operand: H::create_value_operand(value),
            });
        }
    }

    if H::supports_rotation() {
        if let Some(handler_index) = H::try_consume_rotation(ops) {
            return Some(ProcessedInstr {
                handler_index,
                operand: H::create_value_operand(value),
            });
        }
    }

    if let Some(handler_index) = H::try_consume_conversion(ops) {
        return Some(ProcessedInstr {
            handler_index,
            operand: H::create_value_operand(value),
        });
    }

    H::try_consume_arithmetic(ops).map(|handler_index| ProcessedInstr {
        handler_index,
        operand: H::create_value_operand(value),
    })
}

pub fn try_superinstructions_const(
    op: &wasmparser::Operator,
    ops: &mut std::iter::Peekable<wasmparser::OperatorsIteratorWithOffsets<'_>>,
) -> Option<ProcessedInstr> {
    match op {
        wasmparser::Operator::I32Const { value } => {
            handle_const_patterns::<I32Handler>(*value, ops)
        }
        wasmparser::Operator::I64Const { value } => {
            handle_const_patterns::<I64Handler>(*value, ops)
        }
        wasmparser::Operator::F32Const { value } => {
            handle_const_patterns::<F32Handler>(f32::from_bits(value.bits()), ops)
        }
        wasmparser::Operator::F64Const { value } => {
            handle_const_patterns::<F64Handler>(f64::from_bits(value.bits()), ops)
        }
        _ => None,
    }
}
