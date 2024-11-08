use iter_extended::vecmap;

use crate::ssa::{function_builder::FunctionBuilder, ir::function::FunctionId};

use super::{ParsedSsa, ParsedTerminator, ParsedValue, Ssa, SsaError};

impl ParsedSsa {
    pub(crate) fn into_ssa(self) -> Result<Ssa, SsaError> {
        let mut translator = Translator {};
        translator.translate_parsed_ssa(self)
    }
}

struct Translator {}

impl Translator {
    fn translate_parsed_ssa(&mut self, mut parsed_ssa: ParsedSsa) -> Result<Ssa, SsaError> {
        let mut main_function = parsed_ssa.functions.remove(0);
        let main_id = FunctionId::new(0);

        let mut builder = FunctionBuilder::new(main_function.external_name, main_id);
        builder.set_runtime(main_function.runtime_type);

        let entry_block = main_function.blocks.remove(0);
        match entry_block.terminator {
            ParsedTerminator::Return(values) => {
                let return_values = vecmap(values, |value| match value {
                    ParsedValue::NumericConstant { constant, typ } => {
                        builder.numeric_constant(constant, typ)
                    }
                });
                builder.terminate_with_return(return_values);
            }
        }

        Ok(builder.finish())
    }
}
