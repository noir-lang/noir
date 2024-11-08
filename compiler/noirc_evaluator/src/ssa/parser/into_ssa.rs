use std::collections::HashMap;

use im::Vector;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{basic_block::BasicBlockId, function::FunctionId, value::ValueId},
};

use super::{
    Identifier, ParsedBlock, ParsedFunction, ParsedSsa, ParsedTerminator, ParsedValue, Ssa,
    SsaError,
};

impl ParsedSsa {
    pub(crate) fn into_ssa(mut self) -> Result<Ssa, SsaError> {
        let translator = Translator::new(&mut self)?;
        Ok(translator.finish())
    }
}

struct Translator {
    builder: FunctionBuilder,

    /// Maps block names to their IDs
    blocks: HashMap<String, BasicBlockId>,

    /// Maps parameter names to their IDs
    parameters: HashMap<String, ValueId>,
}

impl Translator {
    fn new(parsed_ssa: &mut ParsedSsa) -> Result<Self, SsaError> {
        let main_function = parsed_ssa.functions.remove(0);
        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new(main_function.external_name.clone(), main_id);
        builder.set_runtime(main_function.runtime_type);

        let mut translator = Self { builder, parameters: HashMap::new(), blocks: HashMap::new() };
        translator.translate_function_body(main_function)?;
        Ok(translator)
    }

    fn translate_function_body(&mut self, mut function: ParsedFunction) -> Result<(), SsaError> {
        // First define all blocks so that they are known (a block might jump to a block that comes next)
        let entry_block = function.blocks.remove(0);
        let entry_block_id = self.builder.current_function.entry_block();
        self.blocks.insert(entry_block.name.clone(), entry_block_id);

        for block in &function.blocks {
            let block_id = self.builder.insert_block();
            self.blocks.insert(block.name.clone(), block_id);
        }

        self.translate_block(entry_block)?;

        for block in function.blocks {
            self.translate_block(block)?;
        }

        Ok(())
    }

    fn translate_block(&mut self, block: ParsedBlock) -> Result<(), SsaError> {
        let block_id = self.blocks[&block.name];
        self.builder.switch_to_block(block_id);

        for parameter in block.parameters {
            let parameter_value_id = self.builder.add_block_parameter(block_id, parameter.typ);
            self.parameters.insert(parameter.identifier.name, parameter_value_id);
        }

        match block.terminator {
            ParsedTerminator::Jmp { destination, arguments } => {
                let block_id = self.lookup_block(destination)?;

                let mut translated_arguments = Vec::with_capacity(arguments.len());
                for value in arguments {
                    translated_arguments.push(self.translate_value(value)?);
                }

                self.builder.terminate_with_jmp(block_id, translated_arguments);
            }
            ParsedTerminator::Jmpif { condition, then_block, else_block } => {
                let condition = self.translate_value(condition)?;
                let then_destination = self.lookup_block(then_block)?;
                let else_destination = self.lookup_block(else_block)?;

                self.builder.terminate_with_jmpif(condition, then_destination, else_destination);
            }
            ParsedTerminator::Return(values) => {
                let mut return_values = Vec::with_capacity(values.len());
                for value in values {
                    return_values.push(self.translate_value(value)?);
                }
                self.builder.terminate_with_return(return_values);
            }
        }

        Ok(())
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
        if let Some(value_id) = self.parameters.get(&identifier.name) {
            Ok(*value_id)
        } else {
            Err(SsaError::UnknownVariable(identifier))
        }
    }

    fn lookup_block(&mut self, identifier: Identifier) -> Result<BasicBlockId, SsaError> {
        if let Some(block_id) = self.blocks.get(&identifier.name) {
            Ok(*block_id)
        } else {
            Err(SsaError::UnknownBlock(identifier))
        }
    }

    fn finish(self) -> Ssa {
        self.builder.finish()
    }
}
