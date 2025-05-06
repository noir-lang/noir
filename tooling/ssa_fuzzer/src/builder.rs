#![allow(dead_code)]
use crate::compiler::compile;
use crate::helpers::{id_to_int, u32_to_id_value};
use crate::typed_value::{TypedValue, ValueType};
use acvm::FieldElement;
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_evaluator::ssa::function_builder::FunctionBuilder;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use noirc_evaluator::ssa::ir::function::{Function, RuntimeType};
use noirc_evaluator::ssa::ir::instruction::BinaryOp;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::{NumericType, Type};
use noirc_evaluator::ssa::ir::value::Value;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FuzzerBuilderError {
    #[error("Compilation panicked: {0}")]
    RuntimeError(String),
}

/// Inserts binary instruction with two arguments and returns Id of the result
/// eg add, sub, mul
pub type InstructionWithTwoArgs = fn(&mut FuzzerBuilder, TypedValue, TypedValue) -> TypedValue;

/// Inserts unary instruction with one argument and returns Id of the result
/// eg not, SimpleCast(casts variable to current numeric type)
pub type InstructionWithOneArg = fn(&mut FuzzerBuilder, TypedValue) -> TypedValue;

/// Builder for generating fuzzed SSA functions
/// Contains a FunctionBuilder and tracks the current numeric type being used
pub struct FuzzerBuilder {
    pub(crate) builder: FunctionBuilder,
    pub(crate) numeric_type: NumericType,
    pub(crate) type_: Type,
}

impl FuzzerBuilder {
    /// Creates a new FuzzerBuilder in ACIR context
    pub fn new_acir() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        Self {
            builder,
            numeric_type: NumericType::NativeField,
            type_: Type::Numeric(NumericType::NativeField),
        }
    }

    /// Creates a new FuzzerBuilder in Brillig context
    pub fn new_brillig() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        Self {
            builder,
            numeric_type: NumericType::NativeField,
            type_: Type::Numeric(NumericType::NativeField),
        }
    }

    /// Compiles the built function into a CompiledProgram, to run it with nargo execute
    pub fn compile(self) -> Result<CompiledProgram, FuzzerBuilderError> {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            compile(self.builder, &CompileOptions::default())
        }));
        match result {
            Ok(result) => match result {
                Ok(result) => Ok(result),
                Err(e) => {
                    Err(FuzzerBuilderError::RuntimeError(format!("Compilation error {:?}", e)))
                }
            },
            Err(_) => Err(FuzzerBuilderError::RuntimeError("Compilation panicked".to_string())),
        }
    }

    /// Inserts initial variables of the given type into the function
    pub fn insert_variable(&mut self, variable_type: Type) -> TypedValue {
        let id = self.builder.add_parameter(variable_type.clone());
        TypedValue::new(id, variable_type)
    }

    /// Terminates main function block with the given value
    pub fn finalize_function(&mut self, return_value: TypedValue) {
        self.builder.terminate_with_return(vec![return_value.value_id]);
    }

    fn balance_arithmetic(&mut self, lhs: TypedValue, rhs: TypedValue) -> (TypedValue, TypedValue) {
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "arithmetic") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }

        // cannot cast
        if !lhs.compatible_with(&rhs, "arithmetic") {
            return (lhs.clone(), lhs);
        }

        return (lhs, rhs);
    }

    /// Inserts an add instruction between two values
    pub fn insert_add_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Add { unchecked: false },
            rhs.value_id,
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a subtract instruction between two values
    pub fn insert_sub_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Sub { unchecked: false },
            rhs.value_id,
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a multiply instruction between two values
    pub fn insert_mul_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Mul { unchecked: false },
            rhs.value_id,
        );
        let init_bit_length = match lhs.type_of_variable {
            Type::Numeric(NumericType::NativeField) => 254,
            Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
            Type::Numeric(NumericType::Signed { bit_size }) => bit_size,
            _ => unreachable!("Trying to cast not numeric type"),
        };

        self.builder.insert_range_check(
            res,
            init_bit_length,
            Some("Attempt to multiply with overflow".to_string()),
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a divide instruction between two values
    pub fn insert_div_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Div, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a modulo instruction between two values
    pub fn insert_mod_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let mut lhs = lhs;
        let mut rhs = rhs;
        let (lhs, mut rhs) = match (lhs.supports_mod(), rhs.supports_mod()) {
            (true, true) => self.balance_arithmetic(lhs, rhs),
            (true, false) => self.balance_arithmetic(lhs, rhs),
            (false, true) => self.balance_arithmetic(rhs, lhs),
            _ => {
                // field case, doesn't support mod, cannot balance
                return lhs;
            }
        };
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Mod, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a not instruction for the given value
    pub fn insert_not_instruction(&mut self, lhs: TypedValue) -> TypedValue {
        if !lhs.supports_not() {
            return lhs;
        }
        let res = self.builder.insert_not(lhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a cast instruction
    pub fn insert_cast(&mut self, value: TypedValue, cast_type: ValueType) -> TypedValue {
        if !cast_type.can_be_used_for_casts() {
            return value;
        }

        let init_bit_length = match value.type_of_variable {
            Type::Numeric(NumericType::NativeField) => 254,
            Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
            Type::Numeric(NumericType::Signed { bit_size }) => bit_size,
            _ => unreachable!("Trying to cast not to numeric type"),
        };

        let mut value_id = value.value_id;
        // always truncate, optimizations will eliminate if we truncate to bigger bit_len
        if cast_type.bit_length() != 254 {
            value_id = self.builder.insert_truncate(
                value.value_id,
                cast_type.bit_length(),
                init_bit_length,
            );
        }
        if cast_type.bit_length() > value.to_value_type().bit_length() {
            return value;
        }

        let res = self.builder.insert_cast(value_id, cast_type.to_numeric_type());
        TypedValue::new(res, cast_type.to_ssa_type())
    }

    /// Inserts an equals comparison instruction between two values
    pub fn insert_eq_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Eq, rhs.value_id);
        TypedValue::new(res, ValueType::Boolean.to_ssa_type())
    }

    /// Inserts a less than comparison instruction between two values
    pub fn insert_lt_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_arithmetic(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Lt, rhs.value_id);
        TypedValue::new(res, ValueType::Boolean.to_ssa_type())
    }

    fn balance_types_for_bitwise_op(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> (TypedValue, TypedValue) {
        let mut lhs = lhs;
        let mut rhs = rhs;
        let (lhs, mut rhs) = match (lhs.supports_bitwise(), rhs.supports_bitwise()) {
            (true, true) => (lhs, rhs),
            (true, false) => {
                rhs = self.insert_cast(rhs, lhs.to_value_type());
                (lhs, rhs)
            }
            (false, true) => {
                lhs = self.insert_cast(lhs, rhs.to_value_type());
                (lhs, rhs)
            }
            _ => {
                // field case, doesn't support bitwise, cast to u64
                // TODO(sn): make something smarter
                lhs = self.insert_cast(lhs, ValueType::U64);
                rhs = self.insert_cast(rhs, ValueType::U64);
                (lhs, rhs)
            }
        };
        if !lhs.compatible_with(&rhs, "bitwise") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }

        if !lhs.compatible_with(&rhs, "bitwise") {
            // cannot cast
            return (lhs.clone(), lhs);
        }
        (lhs, rhs)
    }

    /// Inserts a bitwise AND instruction between two values
    pub fn insert_and_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // TODO
        return lhs;
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::And, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise OR instruction between two values
    pub fn insert_or_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // TODO
        return lhs;
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Or, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise XOR instruction between two values    
    pub fn insert_xor_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // TODO
        return lhs;
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Xor, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a left shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shl_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        //TODO
        return lhs;
        // rhs must be 8bit, otherwise compiler will throw panic...
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        let mut lhs = lhs;
        if !lhs.supports_shift() {
            return lhs;
        }
        if !lhs.compatible_with(&rhs, "shift") {
            // if field type, cast to u64
            lhs = self.insert_cast(lhs, ValueType::U64);
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shl, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a right shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shr_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // TODO
        return lhs;
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        let mut lhs = lhs;
        if !lhs.supports_shift() {
            return lhs;
        }
        if !lhs.compatible_with(&rhs, "shift") {
            // if field type, cast to u64
            lhs = self.insert_cast(lhs, ValueType::U64);
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shr, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    pub fn insert_constant(
        &mut self,
        value: impl Into<FieldElement>,
        type_: ValueType,
    ) -> TypedValue {
        let id = self.builder.numeric_constant(value.into(), type_.to_numeric_type());
        TypedValue::new(id, type_.to_ssa_type())
    }
    /// Gets the index of the entry block
    pub fn get_current_block(&mut self) -> BasicBlockId {
        self.builder.get_current_block_index()
    }

    /// Switches the current block to the given block
    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.builder.switch_to_block(block);
    }

    /// Inserts a new basic block and returns its index
    pub fn insert_block(&mut self) -> BasicBlockId {
        self.builder.insert_block()
    }

    /// Inserts a return instruction with the given value
    pub fn insert_return_instruction(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    /// Inserts an unconditional jump to the given block with parameters
    pub fn insert_jmp_instruction(&mut self, destination: BasicBlockId) {
        // we have no arguments to jump to the destination block, we work in single function
        self.builder.terminate_with_jmp(destination, vec![]);
    }

    /// Inserts a conditional jump based on the condition value
    pub fn insert_jmpif_instruction(
        &mut self,
        condition: Id<Value>,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) {
        self.builder.terminate_with_jmpif(condition, then_destination, else_destination);
    }
}
