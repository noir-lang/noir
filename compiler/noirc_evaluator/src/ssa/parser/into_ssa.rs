use std::collections::HashMap;

use im::Vector;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{function::FunctionId, value::ValueId},
};

use super::{ParsedFunction, ParsedSsa, ParsedTerminator, ParsedValue, Ssa, SsaError};

impl ParsedSsa {
    pub(crate) fn into_ssa(mut self) -> Result<Ssa, SsaError> {
        let translator = Translator::new(&mut self)?;
        Ok(translator.finish())
    }
}

struct Translator {
    builder: FunctionBuilder,

    // Maps parameter names to their value IDs
    parameters: HashMap<String, ValueId>,
}

impl Translator {
    fn new(parsed_ssa: &mut ParsedSsa) -> Result<Self, SsaError> {
        let main_function = parsed_ssa.functions.remove(0);
        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new(main_function.external_name.clone(), main_id);
        builder.set_runtime(main_function.runtime_type);

        let mut translator = Self { builder, parameters: HashMap::new() };
        translator.translate_function_body(main_function)?;
        Ok(translator)
    }

    fn translate_function_body(&mut self, mut function: ParsedFunction) -> Result<(), SsaError> {
        let entry_block = function.blocks.remove(0);
        for parameter in entry_block.parameters {
            let parameter_value_id = self.builder.add_parameter(parameter.typ);
            self.parameters.insert(parameter.identifier.name, parameter_value_id);
        }

        match entry_block.terminator {
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
            ParsedValue::Variable(identifier) => {
                if let Some(value_id) = self.parameters.get(&identifier.name) {
                    Ok(*value_id)
                } else {
                    Err(SsaError::UnknownVariable(identifier))
                }
            }
        }
    }

    fn finish(self) -> Ssa {
        self.builder.finish()
    }
}
