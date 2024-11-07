use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::function::{FunctionId, RuntimeType},
};

use super::{Ssa, SsaError};

#[derive(Debug)]
pub(crate) struct ParsedSsa {
    pub(crate) functions: Vec<ParsedFunction>,
}

impl ParsedSsa {
    pub(crate) fn into_ssa(mut self) -> Result<Ssa, SsaError> {
        let mut main_function = self.functions.remove(0);
        let main_id = FunctionId::new(0);

        let mut builder = FunctionBuilder::new(main_function.external_name, main_id);
        builder.set_runtime(main_function.runtime_type);

        let entry_block = main_function.blocks.remove(0);
        match entry_block.terminator {
            ParsedTerminator::Return => {
                builder.terminate_with_return(vec![]);
            }
        }

        Ok(builder.finish())
    }
}

#[derive(Debug)]
pub(crate) struct ParsedFunction {
    pub(crate) runtime_type: RuntimeType,
    pub(crate) external_name: String,
    pub(crate) internal_name: String,
    pub(crate) blocks: Vec<ParsedBlock>,
}

#[derive(Debug)]
pub(crate) struct ParsedBlock {
    pub(crate) name: String,
    pub(crate) instructions: Vec<ParsedInstruction>,
    pub(crate) terminator: ParsedTerminator,
}

#[derive(Debug)]
pub(crate) enum ParsedInstruction {}

#[derive(Debug)]
pub(crate) enum ParsedTerminator {
    Return,
}
