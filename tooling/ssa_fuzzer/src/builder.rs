#![allow(dead_code)]
use crate::compiler::compile_from_builder;
use crate::helpers::{id_to_int, u32_to_id_value};
use crate::typed_value::{TypedValue, ValueType};
use acvm::FieldElement;
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_evaluator::ssa::function_builder::FunctionBuilder;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use noirc_evaluator::ssa::ir::function::{Function, RuntimeType};
use noirc_evaluator::ssa::ir::instruction::{ArrayOffset, BinaryOp};
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
            compile_from_builder(self.builder, &CompileOptions::default())
        }));
        match result {
            Ok(result) => match result {
                Ok(result) => Ok(result),
                Err(_) => Err(FuzzerBuilderError::RuntimeError("Compilation error".to_string())),
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

    /// Inserts an add instruction between two values
    pub fn insert_add_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "add") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
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
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "sub") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
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
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "mul") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Mul { unchecked: false },
            rhs.value_id,
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a divide instruction between two values
    pub fn insert_div_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "div") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Div, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a modulo instruction between two values
    pub fn insert_mod_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let mut lhs = lhs;
        let mut rhs = rhs;
        let (lhs, rhs) = match (lhs.supports_mod(), rhs.supports_mod()) {
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
        // if we cast to lower type, we need to truncate
        let init_bit_length = match value.type_of_variable {
            Type::Numeric(NumericType::NativeField) => 254,
            Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
            Type::Numeric(NumericType::Signed { bit_size }) => bit_size,
            _ => unreachable!("Trying to cast not numeric type"),
        };

        let value_id = if init_bit_length > cast_type.bit_length() {
            self.builder.insert_truncate(value.value_id, cast_type.bit_length(), init_bit_length)
        } else {
            value.value_id
        };

        let res = self.builder.insert_cast(value_id, cast_type.to_numeric_type());
        TypedValue::new(res, cast_type.to_ssa_type())
    }

    /// Inserts an equals comparison instruction between two values
    pub fn insert_eq_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "eq") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Eq, rhs.value_id);
        TypedValue::new(res, ValueType::Boolean.to_ssa_type())
    }

    /// Inserts a less than comparison instruction between two values
    pub fn insert_lt_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let mut rhs = rhs;
        if !lhs.compatible_with(&rhs, "lt") {
            rhs = self.insert_cast(rhs, lhs.to_value_type());
        }
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
        let (lhs, rhs) = match (lhs.supports_bitwise(), rhs.supports_bitwise()) {
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
        (lhs, rhs)
    }

    /// Inserts a bitwise AND instruction between two values
    pub fn insert_and_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::And, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise OR instruction between two values
    pub fn insert_or_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Or, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise XOR instruction between two values    
    pub fn insert_xor_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let (lhs, rhs) = self.balance_types_for_bitwise_op(lhs, rhs);
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Xor, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a left shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shl_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // rhs must be 8bit, otherwise compiler will throw panic...
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        let mut lhs = lhs;
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
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        let mut lhs = lhs;
        if !lhs.compatible_with(&rhs, "shift") {
            // if field type, cast to u64
            lhs = self.insert_cast(lhs, ValueType::U64);
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shr, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Creates an array with the given Ids of values
    pub fn insert_make_array(&mut self, elements: Vec<u32>) -> Id<Value> {
        let mut elems = Vec::new();
        for elem in elements.clone() {
            elems.push(u32_to_id_value(elem));
        }
        let types = vec![self.type_.clone()];

        self.builder.insert_make_array(
            im::Vector::from(elems),
            Type::Array(Arc::new(types), elements.len() as u32),
        )
    }

    /// Gets an element from an array at the given index
    pub fn insert_array_get(&mut self, array: Id<Value>, index: u32) -> Id<Value> {
        let index_var =
            self.builder.numeric_constant(index, NumericType::Unsigned { bit_size: 32 });

        let offset = ArrayOffset::None;
        self.builder.insert_array_get(array, index_var, offset, self.type_.clone())
    }

    pub fn insert_constant(
        &mut self,
        value: impl Into<FieldElement>,
        type_: ValueType,
    ) -> TypedValue {
        let id = self.builder.numeric_constant(value.into(), type_.to_numeric_type());
        TypedValue::new(id, type_.to_ssa_type())
    }

    /// Sets an element in an array at the given index
    pub fn insert_array_set(
        &mut self,
        array: Id<Value>,
        index: u32,
        value: Id<Value>,
    ) -> Id<Value> {
        let index_var =
            self.builder.numeric_constant(index, NumericType::Unsigned { bit_size: 32 });

        let mutable = false;
        let offset = ArrayOffset::None;
        self.builder.insert_array_set(array, index_var, value, mutable, offset)
    }

    /// Gets the index of the entry block
    pub fn get_entry_block_index(&mut self) -> u32 {
        id_to_int(self.builder.get_current_block_index())
    }

    /// Switches the current block to the given block
    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.builder.switch_to_block(block);
    }

    /// Inserts a new basic block and returns its index
    pub fn insert_block(&mut self) -> u32 {
        let id = self.builder.insert_block();
        id_to_int(id)
    }

    /// Inserts a return instruction with the given value
    pub fn insert_return_instruction(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    /// Inserts an unconditional jump to the given block with parameters
    pub fn insert_jmp_instruction(&mut self, destination: BasicBlockId, params: Vec<Id<Value>>) {
        // we have no arguments to jump to the destination block, we work in single function
        self.builder.terminate_with_jmp(destination, params);
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

    /// Creates a numeric constant with the given value
    pub fn numeric_constant(&mut self, value: impl Into<FieldElement>) -> Id<Value> {
        self.builder.numeric_constant(value.into(), self.numeric_type)
    }
}
