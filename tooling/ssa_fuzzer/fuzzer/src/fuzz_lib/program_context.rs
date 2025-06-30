use super::function_context::{FunctionData, FuzzerFunctionContext};
use super::options::FunctionContextOptions;
use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError},
    typed_value::ValueType,
};
use noirc_driver::CompiledProgram;

pub(crate) struct FuzzerProgramContext {
    acir_builder: FuzzerBuilder,
    brillig_builder: FuzzerBuilder,
    program_context_options: FunctionContextOptions, // TODO
    is_constant: bool,
}

impl FuzzerProgramContext {
    pub(crate) fn new(program_context_options: FunctionContextOptions) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self { acir_builder, brillig_builder, program_context_options, is_constant: false }
    }

    pub(crate) fn new_constant_context(program_context_options: FunctionContextOptions) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self { acir_builder, brillig_builder, program_context_options, is_constant: true }
    }

    pub(crate) fn process_function(
        &mut self,
        data: &FunctionData,
        types: &Vec<ValueType>,
        values: &Vec<impl Into<FieldElement> + Clone>,
    ) {
        // TODO SWITCH FUNCTIONS
        let mut function_context = if self.is_constant {
            FuzzerFunctionContext::new_constant_context(
                values.to_vec(),
                types.to_vec(),
                &data.blocks,
                self.program_context_options.clone(),
                &mut self.acir_builder,
                &mut self.brillig_builder,
            )
        } else {
            FuzzerFunctionContext::new(
                types.to_vec(),
                &data.blocks,
                self.program_context_options.clone(),
                &mut self.acir_builder,
                &mut self.brillig_builder,
            )
        };
        for command in &data.commands {
            function_context.process_fuzzer_command(command);
        }
        function_context.finalize(data.return_instruction_block_idx)
    }

    /// Returns programs for ACIR and Brillig
    pub(crate) fn get_programs(
        self,
    ) -> (Result<CompiledProgram, FuzzerBuilderError>, Result<CompiledProgram, FuzzerBuilderError>)
    {
        (
            self.acir_builder.compile(self.program_context_options.compile_options.clone()),
            self.brillig_builder.compile(self.program_context_options.compile_options),
        )
    }

    /// Returns witnesses for ACIR and Brillig
    /// If program does not have any instructions, it terminated with the last witness
    /// Resulting WitnessStack of programs contains only variables and return value
    /// If we inserted some instructions, WitnessStack contains return value, so we return the last one
    /// If we are checking constant folding, the witness stack will only contain the return value, so we return Witness(0)
    pub(crate) fn get_return_witnesses(&self) -> (Witness, Witness) {
        if self.is_constant {
            (Witness(0), Witness(0))
        } else {
            (
                Witness(super::NUMBER_OF_VARIABLES_INITIAL),
                Witness(super::NUMBER_OF_VARIABLES_INITIAL),
            )
        }
    }
}
