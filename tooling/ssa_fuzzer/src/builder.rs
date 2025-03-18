use crate::compiler::compile;
use crate::config::NUMBER_OF_VARIABLES_INITIAL;
use crate::helpers;
use acvm::FieldElement;
use noirc_driver::{CompileError, CompileOptions, CompiledProgram};
use noirc_evaluator::ssa::function_builder::FunctionBuilder;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use noirc_evaluator::ssa::ir::function::{Function, RuntimeType};
use noirc_evaluator::ssa::ir::instruction::BinaryOp;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::{NumericType, Type};
use noirc_evaluator::ssa::ir::value::Value;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use std::sync::Arc;

/// Builder for generating fuzzed SSA functions
/// Contains a FunctionBuilder and tracks the current numeric type being used
pub struct FuzzerBuilder {
    builder: FunctionBuilder,
    numeric_type: NumericType,
    type_: Type,
}

impl FuzzerBuilder {
    /// Creates a new FuzzerBuilder in ACIR context
    pub fn new_acir() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        return Self {
            builder,
            numeric_type: NumericType::NativeField,
            type_: Type::Numeric(NumericType::NativeField),
        };
    }

    /// Creates a new FuzzerBuilder in Brillig context
    pub fn new_brillig() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        return Self {
            builder,
            numeric_type: NumericType::NativeField,
            type_: Type::Numeric(NumericType::NativeField),
        };
    }

    /// Compiles the built function into a CompiledProgram, to run it with nargo execute
    pub fn compile(self) -> Result<CompiledProgram, CompileError> {
        compile(self.builder, &CompileOptions::default())
    }

    /// Inserts initial variables of the given type into the function
    pub fn insert_variables(&mut self, variable_type: Type) {
        for _ in 0..NUMBER_OF_VARIABLES_INITIAL {
            self.builder.add_parameter(variable_type.clone());
        }
        self.type_ = variable_type.clone();
        match variable_type {
            Type::Numeric(numeric_type) => {
                self.numeric_type = numeric_type;
            }
            _ => {
                panic!("Unsupported variable type");
            }
        }
    }

    /// Terminates main function block with the given value
    pub fn finalize_function(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    /// Inserts an add instruction between two values
    pub fn insert_add_instruction_checked(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Add { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_add_instruction_unchecked(
        &mut self,
        lhs: Id<Value>,
        rhs: Id<Value>,
    ) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Add { unchecked: true }, rhs);
        return result;
    }

    /// Inserts a subtract instruction between two values
    pub fn insert_sub_instruction_checked(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Sub { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_sub_instruction_unchecked(
        &mut self,
        lhs: Id<Value>,
        rhs: Id<Value>,
    ) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Sub { unchecked: true }, rhs);
        return result;
    }

    /// Inserts a multiply instruction between two values
    pub fn insert_mul_instruction_checked(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Mul { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_mul_instruction_unchecked(
        &mut self,
        lhs: Id<Value>,
        rhs: Id<Value>,
    ) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, rhs);
        return result;
    }

    /// Inserts a divide instruction between two values
    pub fn insert_div_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Div, rhs);
        return result;
    }

    /// Inserts a modulo instruction between two values
    pub fn insert_mod_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Mod, rhs);
        return result;
    }

    /// Inserts a not instruction for the given value
    pub fn insert_not_instruction(&mut self, lhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_not(lhs);
        return result;
    }

    /// Inserts a cast instruction to the current numeric type
    pub fn insert_simple_cast(&mut self, value: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_cast(value, self.numeric_type.clone());
        return result;
    }

    /// Inserts a cast to a larger bit size and back to original type
    pub fn insert_cast_bigger_and_back(&mut self, value: Id<Value>, size: u32) -> Id<Value> {
        // SSA supports casts to really big sizes, but the program runs for too long
        // it cannot be introduced with just nargo and noir compiler, so we just skip it
        if size > 127 {
            return value;
        }
        match self.numeric_type {
            NumericType::Signed { bit_size: _ } => {
                let res1 = self.builder.insert_cast(value, NumericType::Signed { bit_size: size });
                let result = self.insert_simple_cast(res1);
                return result;
            }
            NumericType::Unsigned { bit_size: _ } => {
                let res1 =
                    self.builder.insert_cast(value, NumericType::Unsigned { bit_size: size });
                let result = self.insert_simple_cast(res1);
                return result;
            }
            NumericType::NativeField => {
                return value;
            }
        }
    }

    /// Inserts an equals comparison instruction between two values
    pub fn insert_eq_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Eq, rhs);
        let result = self.insert_simple_cast(res1);
        return result;
    }

    /// Inserts a less than comparison instruction between two values
    pub fn insert_lt_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Lt, rhs);
        let result = self.insert_simple_cast(res1);
        return result;
    }

    /// Inserts a bitwise AND instruction between two values
    pub fn insert_and_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::And, rhs);
        return result;
    }

    /// Inserts a bitwise OR instruction between two values
    pub fn insert_or_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Or, rhs);
        return result;
    }

    /// Inserts a bitwise XOR instruction between two values
    pub fn insert_xor_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Xor, rhs);
        return result;
    }

    /// Inserts a left shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shl_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        // rhs must be 8bit, otherwise compiler will throw panic...
        match self.numeric_type {
            NumericType::Signed { bit_size: _ } => {
                let rhs_value = self.builder.insert_cast(rhs, NumericType::Signed { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shl, rhs_value);
                return result;
            }
            NumericType::Unsigned { bit_size: _ } => {
                let rhs_value =
                    self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shl, rhs_value);
                return result;
            }
            _ => {
                // field case, doesnt support shift
                return lhs;
            }
        }
    }

    /// Inserts a right shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shr_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        match self.numeric_type {
            NumericType::Signed { bit_size: _ } => {
                let rhs_value =
                    self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shr, rhs_value);
                return result;
            }
            NumericType::Unsigned { bit_size: _ } => {
                let rhs_value =
                    self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shr, rhs_value);
                return result;
            }
            _ => {
                // field case, doesnt support shift
                return lhs;
            }
        }
    }

    /// Creates an array with the given Ids of values
    pub fn insert_make_array(&mut self, elements: Vec<u32>) -> Id<Value> {
        let mut elems = Vec::new();
        for elem in elements.clone() {
            elems.push(helpers::u32_to_id_value(elem));
        }
        let types = vec![self.type_.clone(); elements.len()];
        let result = self.builder.insert_make_array(
            im::Vector::from(elems),
            Type::Array(Arc::new(types), elements.len() as u32),
        );
        return result;
    }

    /// Gets an element from an array at the given index
    pub fn insert_array_get(&mut self, array: Id<Value>, index: u32) -> Id<Value> {
        let index_var =
            self.builder.numeric_constant(index, NumericType::Unsigned { bit_size: 32 });
        let result = self.builder.insert_array_get(array, index_var, self.type_.clone());
        return result;
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
        let result = self.builder.insert_array_set(array, index_var, value);
        return result;
    }

    /// Gets the index of the entry block
    pub fn get_entry_block_index(&mut self) -> u32 {
        return helpers::id_to_int(self.builder.get_current_block_index());
    }

    /// Switches the current block to the given block
    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.builder.switch_to_block(block);
    }

    /// Inserts a new basic block and returns its index
    pub fn insert_block(&mut self) -> u32 {
        let id = self.builder.insert_block();
        return helpers::id_to_int(id);
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
        self.builder.numeric_constant(value.into(), self.numeric_type.clone())
    }
}
