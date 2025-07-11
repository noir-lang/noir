use crate::typed_value::{TypedValue, ValueType};
use acvm::FieldElement;
use noir_ssa_executor::compiler::compile_from_ssa;
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
}

impl FuzzerBuilder {
    /// Creates a new FuzzerBuilder in ACIR context
    pub fn new_acir() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        Self { builder }
    }

    /// Creates a new FuzzerBuilder in Brillig context
    pub fn new_brillig() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        Self { builder }
    }

    /// Compiles the built function into a CompiledProgram, to run it with nargo execute
    pub fn compile(
        self,
        compile_options: CompileOptions,
    ) -> Result<CompiledProgram, FuzzerBuilderError> {
        let ssa = self.builder.finish();
        let result =
            std::panic::catch_unwind(AssertUnwindSafe(|| compile_from_ssa(ssa, &compile_options)));
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
    pub fn finalize_function(&mut self, return_value: &TypedValue) {
        self.builder.terminate_with_return(vec![return_value.value_id]);
    }

    /// Inserts an add instruction between two values
    pub fn insert_add_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
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
        if lhs.to_value_type().bit_length() == 254 {
            return TypedValue::new(res, lhs.type_of_variable);
        }
        let bit_size = lhs.to_value_type().bit_length();
        let res = self.builder.insert_truncate(res, bit_size, bit_size * 2);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a divide instruction between two values
    pub fn insert_div_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Div, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a modulo instruction between two values
    pub fn insert_mod_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        if !lhs.supports_mod() {
            return lhs;
        }
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
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Eq, rhs.value_id);
        TypedValue::new(res, ValueType::Boolean.to_ssa_type())
    }

    /// Inserts a less than comparison instruction between two values
    pub fn insert_lt_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Lt, rhs.value_id);
        TypedValue::new(res, ValueType::Boolean.to_ssa_type())
    }

    /// Inserts a bitwise AND instruction between two values
    pub fn insert_and_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        if !lhs.supports_bitwise() {
            return lhs;
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::And, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise OR instruction between two values
    pub fn insert_or_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        if !lhs.supports_bitwise() {
            return lhs;
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Or, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise XOR instruction between two values
    pub fn insert_xor_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        if !lhs.supports_bitwise() {
            return lhs;
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Xor, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a left shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shl_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // rhs must be 8bit, otherwise compiler will throw panic...
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        if !lhs.supports_shift() {
            return lhs;
        }
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shl, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a right shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shr_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        // TODO(sn): make something smarter than forcing rhs to be u8 and casting to u64 on field
        let rhs = self.insert_cast(rhs, ValueType::U8);
        if !lhs.supports_shift() {
            return lhs;
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

    pub fn insert_constrain(&mut self, lhs: TypedValue, rhs: TypedValue) {
        self.builder.insert_constrain(lhs.value_id, rhs.value_id, None);
    }

    /// Gets the index of the entry block
    pub fn get_current_block(&self) -> BasicBlockId {
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

    /// Inserts a new parameter to the current SSA block and returns its value
    ///
    /// b0() -> b0(new_parameter: parameter_type)
    pub fn add_block_parameter(&mut self, block: BasicBlockId, typ: ValueType) -> TypedValue {
        let id = self.builder.add_block_parameter(block, typ.to_ssa_type());
        TypedValue::new(id, typ.to_ssa_type())
    }

    /// Inserts a return instruction with the given value
    pub fn insert_return_instruction(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    /// Inserts an unconditional jump to the given block with parameters
    pub fn insert_jmp_instruction(
        &mut self,
        destination: BasicBlockId,
        arguments: Vec<TypedValue>,
    ) {
        self.builder.terminate_with_jmp(
            destination,
            arguments.into_iter().map(|arg| arg.value_id).collect(),
        );
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

    pub fn insert_add_to_memory(&mut self, lhs: TypedValue) -> TypedValue {
        let memory_address = self.builder.insert_allocate(lhs.type_of_variable.clone());
        self.builder.insert_store(memory_address, lhs.value_id);
        TypedValue::new(memory_address, lhs.type_of_variable)
    }

    pub fn insert_load_from_memory(&mut self, memory_addr: TypedValue) -> TypedValue {
        let res =
            self.builder.insert_load(memory_addr.value_id, memory_addr.clone().type_of_variable);
        TypedValue::new(res, memory_addr.type_of_variable.clone())
    }

    pub fn insert_set_to_memory(&mut self, memory_addr: TypedValue, value: TypedValue) {
        self.builder.insert_store(memory_addr.value_id, value.value_id);
    }
}
