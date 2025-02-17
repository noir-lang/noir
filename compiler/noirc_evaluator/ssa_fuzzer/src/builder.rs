use crate::compiler::compile;
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
}

impl FuzzerBuilder {
    pub fn new_acir() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        return Self { builder, numeric_type: NumericType::NativeField };
    }

    pub fn new_brillig() -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        return Self { builder, numeric_type: NumericType::NativeField };
    }

    pub fn compile(self) -> Result<CompiledProgram, CompileError> {
        compile(self.builder, &CompileOptions::default())
    }

    pub fn insert_variables(&mut self, variable_type: Type) {
        for _ in 0..NUMBER_OF_VARIABLES_INITIAL {
            self.builder.add_parameter(variable_type.clone());
        }
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

    pub fn insert_eq_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Eq, rhs);
        let result = self.builder.insert_cast(res1, self.numeric_type.clone());
        return result;
    }

    pub fn insert_lt_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let res1 = self.builder.insert_binary(lhs, BinaryOp::Lt, rhs);
        let result = self.builder.insert_cast(res1, self.numeric_type.clone());
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
        let result = self.builder.insert_binary(lhs, BinaryOp::Shl, rhs);
        return result;
    }

    pub fn insert_shr_instruction(&mut self, lhs: Id<Value>, rhs: Id<Value>) -> Id<Value> {
        let result = self.builder.insert_binary(lhs, BinaryOp::Shr, rhs);
        return result;
    }
}