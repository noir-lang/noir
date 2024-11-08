use std::collections::HashMap;

use im::Vector;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{basic_block::BasicBlockId, function::FunctionId, value::ValueId},
};

use super::{
    Identifier, ParsedBlock, ParsedFunction, ParsedInstruction, ParsedSsa, ParsedTerminator,
    ParsedValue, RuntimeType, Ssa, SsaError, Type,
};

impl ParsedSsa {
    pub(crate) fn into_ssa(mut self) -> Result<Ssa, SsaError> {
        let mut translator = Translator::new(&mut self)?;

        for function in self.functions {
            translator.translate_function(function)?;
        }

        Ok(translator.finish())
    }
}

struct Translator {
    builder: FunctionBuilder,

    /// Maps function names to their ID and types
    functions: HashMap<String, (FunctionId, Vec<Type>)>,

    /// Maps block names to their IDs
    blocks: HashMap<FunctionId, HashMap<String, BasicBlockId>>,

    /// Maps variable names to their IDs
    variables: HashMap<FunctionId, HashMap<String, ValueId>>,
}

impl Translator {
    fn new(parsed_ssa: &mut ParsedSsa) -> Result<Self, SsaError> {
        let main_function = parsed_ssa.functions.remove(0);
        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new(main_function.external_name.clone(), main_id);
        builder.set_runtime(main_function.runtime_type);

        // Map function names to their IDs so calls can be resolved
        let mut function_id_counter = 1;
        let mut functions = HashMap::new();
        for function in &parsed_ssa.functions {
            let function_id = FunctionId::new(function_id_counter);
            function_id_counter += 1;

            functions.insert(
                function.internal_name.clone(),
                (function_id, function.return_types.clone()),
            );
        }

        let mut translator =
            Self { builder, functions, variables: HashMap::new(), blocks: HashMap::new() };
        translator.translate_function_body(main_function)?;

        Ok(translator)
    }

    fn translate_function(&mut self, function: ParsedFunction) -> Result<(), SsaError> {
        let (function_id, _) = self.functions[&function.internal_name];
        let external_name = function.external_name.clone();

        match function.runtime_type {
            RuntimeType::Acir(inline_type) => {
                self.builder.new_function(external_name, function_id, inline_type);
            }
            RuntimeType::Brillig(inline_type) => {
                self.builder.new_brillig_function(external_name, function_id, inline_type);
            }
        }

        self.translate_function_body(function)
    }

    fn translate_function_body(&mut self, function: ParsedFunction) -> Result<(), SsaError> {
        // First define all blocks so that they are known (a block might jump to a block that comes next)
        for (index, block) in function.blocks.iter().enumerate() {
            // The first block is the entry block and it was automatically created by the builder
            let block_id = if index == 0 {
                self.builder.current_function.entry_block()
            } else {
                self.builder.insert_block()
            };
            let entry = self.blocks.entry(self.current_function_id()).or_default();
            entry.insert(block.name.clone(), block_id);
        }

        for block in function.blocks {
            self.translate_block(block)?;
        }

        Ok(())
    }

    fn translate_block(&mut self, block: ParsedBlock) -> Result<(), SsaError> {
        let block_id = self.blocks[&self.current_function_id()][&block.name];
        self.builder.switch_to_block(block_id);

        for parameter in block.parameters {
            let parameter_value_id = self.builder.add_block_parameter(block_id, parameter.typ);
            let entry = self.variables.entry(self.current_function_id()).or_default();
            entry.insert(parameter.identifier.name, parameter_value_id);
        }

        for instruction in block.instructions {
            self.translate_instruction(instruction)?;
        }

        match block.terminator {
            ParsedTerminator::Jmp { destination, arguments } => {
                let block_id = self.lookup_block(destination)?;
                let arguments = self.translate_values(arguments)?;
                self.builder.terminate_with_jmp(block_id, arguments);
            }
            ParsedTerminator::Jmpif { condition, then_block, else_block } => {
                let condition = self.translate_value(condition)?;
                let then_destination = self.lookup_block(then_block)?;
                let else_destination = self.lookup_block(else_block)?;
                self.builder.terminate_with_jmpif(condition, then_destination, else_destination);
            }
            ParsedTerminator::Return(values) => {
                let return_values = self.translate_values(values)?;
                self.builder.terminate_with_return(return_values);
            }
        }

        Ok(())
    }

    fn translate_instruction(&mut self, instruction: ParsedInstruction) -> Result<(), SsaError> {
        match instruction {
            ParsedInstruction::Call { targets, function, arguments } => {
                let (function_id, return_types) = self.lookup_function(function)?;
                let result_types = return_types.to_vec();

                let function_id = self.builder.import_function(function_id);
                let arguments = self.translate_values(arguments)?;

                let current_function_id = self.current_function_id();
                let value_ids = self.builder.insert_call(function_id, arguments, result_types);

                if value_ids.len() != targets.len() {
                    return Err(SsaError::MismatchedReturnValues {
                        returns: targets,
                        expected: value_ids.len(),
                    });
                }

                for (target, value_id) in targets.into_iter().zip(value_ids.iter()) {
                    let entry = self.variables.entry(current_function_id).or_default();
                    entry.insert(target.name, *value_id);
                }
            }
            ParsedInstruction::Cast { target, lhs, typ } => {
                let lhs = self.translate_value(lhs)?;
                let value_id = self.builder.insert_cast(lhs, typ);
                let entry = self.variables.entry(self.current_function_id()).or_default();
                entry.insert(target.name, value_id);
            }
        }

        Ok(())
    }

    fn translate_values(&mut self, values: Vec<ParsedValue>) -> Result<Vec<ValueId>, SsaError> {
        let mut translated_values = Vec::with_capacity(values.len());
        for value in values {
            translated_values.push(self.translate_value(value)?);
        }
        Ok(translated_values)
    }

    fn translate_value(&mut self, value: ParsedValue) -> Result<ValueId, SsaError> {
        match value {
            ParsedValue::NumericConstant { constant, typ } => {
                Ok(self.builder.numeric_constant(constant, typ))
            }
            ParsedValue::Array { values, typ } => {
                let mut translated_values = Vector::new();
                for value in values {
                    translated_values.push_back(self.translate_value(value)?);
                }
                Ok(self.builder.array_constant(translated_values, typ))
            }
            ParsedValue::Variable(identifier) => self.lookup_variable(identifier),
        }
    }

    fn lookup_variable(&mut self, identifier: Identifier) -> Result<ValueId, SsaError> {
        if let Some(value_id) = self.variables[&self.current_function_id()].get(&identifier.name) {
            Ok(*value_id)
        } else {
            Err(SsaError::UnknownVariable(identifier))
        }
    }

    fn lookup_block(&mut self, identifier: Identifier) -> Result<BasicBlockId, SsaError> {
        if let Some(block_id) = self.blocks[&self.current_function_id()].get(&identifier.name) {
            Ok(*block_id)
        } else {
            Err(SsaError::UnknownBlock(identifier))
        }
    }

    fn lookup_function(
        &mut self,
        identifier: Identifier,
    ) -> Result<(FunctionId, &[Type]), SsaError> {
        if let Some((function_id, types)) = self.functions.get(&identifier.name) {
            Ok((*function_id, types))
        } else {
            Err(SsaError::UnknownFunction(identifier))
        }
    }

    fn finish(self) -> Ssa {
        self.builder.finish()
    }

    fn current_function_id(&self) -> FunctionId {
        self.builder.current_function.id()
    }
}
