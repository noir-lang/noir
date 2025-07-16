use std::collections::BTreeMap;

use super::function_context::{FunctionData, FuzzerFunctionContext};
use super::instruction::{FunctionSignature, InstructionBlock};
use super::options::FunctionContextOptions;
use acvm::FieldElement;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError},
    typed_value::ValueType,
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::{function::Function, map::Id};

struct StoredFunction {
    id: Id<Function>,
    function: FunctionData,
    types: Vec<ValueType>,
}

/// FuzzerProgramContext is a context for storing and processing SSA functions
pub(crate) struct FuzzerProgramContext {
    /// Builder for ACIR program
    acir_builder: FuzzerBuilder,
    /// Builder for Brillig program
    brillig_builder: FuzzerBuilder,
    /// Options for the program context
    program_context_options: FunctionContextOptions, // TODO
    /// Whether the program is executed in constants
    is_constant: bool,
    /// Function signatures
    function_signatures: BTreeMap<Id<Function>, FunctionSignature>,
    /// Stored functions
    stored_functions: Vec<StoredFunction>,
    /// Current function id
    current_function_id: Id<Function>,
    /// Instruction blocks
    instruction_blocks: Vec<InstructionBlock>,
    /// Main initialized
    is_main_initialized: bool,
    /// Values of the inputs
    ///
    /// Used for the constant mode (to replace variables in the main function with constants)
    values: Vec<FieldElement>,
}

impl FuzzerProgramContext {
    /// Creates a new FuzzerProgramContext
    pub(crate) fn new(
        program_context_options: FunctionContextOptions,
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
    ) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self {
            acir_builder,
            brillig_builder,
            program_context_options,
            is_constant: false,
            function_signatures: BTreeMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
            instruction_blocks,
            is_main_initialized: false,
            values,
        }
    }

    /// Creates a new FuzzerProgramContext where all inputs are constants
    pub(crate) fn new_constant_context(
        program_context_options: FunctionContextOptions,
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
    ) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self {
            acir_builder,
            brillig_builder,
            program_context_options,
            is_constant: true,
            function_signatures: BTreeMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
            instruction_blocks,
            is_main_initialized: false,
            values,
        }
    }

    /// Stores function and its signature
    pub(crate) fn process_function(&mut self, function: FunctionData, types: Vec<ValueType>) {
        let signature =
            FunctionSignature { input_types: types.clone(), return_type: function.return_type };
        self.function_signatures.insert(self.current_function_id, signature);
        let stored_function = StoredFunction { id: self.current_function_id, function, types };
        self.stored_functions.push(stored_function);
        self.current_function_id = Id::new(self.current_function_id.to_u32() + 1);
    }

    /// Creates new ACIR and Brillig functions for each of stored functions and finalizes them
    pub(crate) fn finalize_program(&mut self) {
        for i in 0..self.stored_functions.len() {
            let stored_function = &self.stored_functions[i];
            // use only functions defined after this one to avoid recursion
            let defined_functions: BTreeMap<Id<Function>, FunctionSignature> = self
                .function_signatures
                .clone()
                .into_iter()
                .filter(|func_id| func_id.0.to_u32() > stored_function.id.to_u32())
                .collect();
            let mut function_context = if self.is_constant && !self.is_main_initialized {
                FuzzerFunctionContext::new_constant_context(
                    self.values
                        .iter()
                        .zip(stored_function.types.iter())
                        .map(|(value, type_)| (*value, *type_))
                        .collect(),
                    &self.instruction_blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type,
                    defined_functions,
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                )
            } else {
                FuzzerFunctionContext::new(
                    stored_function.types.to_vec(),
                    &self.instruction_blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type,
                    defined_functions,
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                )
            };
            self.is_main_initialized = true;
            for command in &stored_function.function.commands {
                function_context.process_fuzzer_command(command);
            }
            function_context.finalize(stored_function.function.return_instruction_block_idx);
            // do not create new function if it's last one
            if i != self.stored_functions.len() - 1 {
                let current_id = stored_function.id;
                let new_id = Id::<Function>::new(current_id.to_u32() + 1);
                self.acir_builder.new_acir_function(format!("f{}", new_id.to_u32()), new_id);
                self.brillig_builder.new_brillig_function(format!("f{}", new_id.to_u32()), new_id);
            }
        }
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
}
