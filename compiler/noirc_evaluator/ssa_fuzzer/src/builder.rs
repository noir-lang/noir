use std::sync::Arc;
use crate::compiler::compile;
use crate::helpers;
use noirc_evaluator::ssa::ir::types::{NumericType, Type};
use noirc_evaluator::ssa::function_builder::FunctionBuilder;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::Value;
use noirc_evaluator::ssa::ir::function::{Function, RuntimeType};
use noirc_evaluator::ssa::ir::instruction::BinaryOp;
use noirc_driver::{CompileOptions, CompiledProgram, CompileError};
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use crate::config::NUMBER_OF_VARIABLES_INITIAL;
pub struct FuzzerBuilder {
    builder: FunctionBuilder,
    numeric_type: NumericType,
    type_: Type,
}

impl FuzzerBuilder {
    pub fn new_acir() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        return Self { builder, numeric_type: NumericType::NativeField, type_: Type::Numeric(NumericType::NativeField) };
    }

    pub fn new_brillig() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        return Self { builder, numeric_type: NumericType::NativeField, type_: Type::Numeric(NumericType::NativeField) };
    }

    pub fn compile(self) -> Result<CompiledProgram, CompileError> {
        compile(self.builder, &CompileOptions::default())
    }

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

    pub fn finalize_function(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    pub fn insert_add_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Add { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_sub_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Sub { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_mul_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Mul { unchecked: false }, rhs);
        return result;
    }

    pub fn insert_div_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Div, rhs);
        return result;
    }

    pub fn insert_mod_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Mod, rhs);
        return result;
    }

    pub fn insert_not_instruction(&mut self, lhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_not(lhs);
        return result;
    }

    pub fn insert_simple_cast(&mut self, value: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_cast(value, self.numeric_type.clone());
        return result;
    }

    pub fn insert_cast_bigger_and_back(&mut self, value: Id<Value>, size: u32) -> Id<Value> {
        // TODO: this is a hack to avoid timeouts
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
                let res1 = self.builder.insert_cast(value, NumericType::Unsigned { bit_size: size });
                let result = self.insert_simple_cast(res1);
                return result;
            }
            NumericType::NativeField => {
                return value;
            }

        }
    }

    pub fn insert_eq_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Eq, rhs);
        let result = self.insert_simple_cast(res1);
        return result;
    }

    pub fn insert_lt_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Lt, rhs);
        let result = self.insert_simple_cast(res1);
        return result;
    }

    pub fn insert_and_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::And, rhs);
        return result;
    }

    pub fn insert_or_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Or, rhs);
        return result;
    }

    pub fn insert_xor_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Xor, rhs);
        return result;
    }

    pub fn insert_shl_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        // rhs must be 8bit 
        match self.numeric_type {
            NumericType::Signed { bit_size: _ }  => {
                let rhs_value = self.builder.insert_cast(rhs, NumericType::Signed { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shl, rhs_value);
                return result;
            }
            NumericType::Unsigned { bit_size: _ } => {
                let rhs_value = self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shl, rhs_value);
                return result;
            }
            _ => {
                // field case
                return lhs;
            }
        }
    }

    pub fn insert_shr_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        match self.numeric_type {
            NumericType::Signed { bit_size: _ }  => {
                let rhs_value = self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shr, rhs_value);
                return result;
            }
            NumericType::Unsigned { bit_size: _ } => {
                let rhs_value = self.builder.insert_cast(rhs, NumericType::Unsigned { bit_size: 8 });
                let result = self.builder.insert_binary(lhs, BinaryOp::Shr, rhs_value);
                return result;
            }
            _ => {
                // field case
                return lhs;
            }
        }
    }

    pub fn insert_make_array(&mut self, elements: Vec<u32>) -> Id<Value> {
        let mut elems = Vec::new();
        for elem in elements.clone() {
            elems.push(helpers::u32_to_id(elem));
        }
        let types = vec![self.type_.clone(); elements.len()];
        let result = self.builder.insert_make_array(
            im::Vector::from(elems), 
            Type::Array(Arc::new(types), elements.len() as u32)
        );
        return result;
    }

    pub fn insert_array_get(&mut self, array: Id<Value>, index: u32) -> Id<Value> {
        let index_var = self.builder.numeric_constant(index, NumericType::Unsigned { bit_size: 32 });
        let result = self.builder.insert_array_get(array, index_var, self.type_.clone());
        return result;
    }

    pub fn insert_array_set(&mut self, array: Id<Value>, index: u32, value: Id<Value>) -> Id<Value> {
        let index_var = self.builder.numeric_constant(index, NumericType::Unsigned { bit_size: 32 });
        let result = self.builder.insert_array_set(array, index_var, value);
        return result;
    }
}