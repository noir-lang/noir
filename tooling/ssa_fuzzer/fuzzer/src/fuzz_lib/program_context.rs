use std::collections::HashMap;

use super::function_context::{FunctionData, FuzzerFunctionContext};
use super::instruction::FunctionSignature;
use super::options::FunctionContextOptions;
use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError},
    typed_value::ValueType,
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::{function::Function, map::Id};

struct StoredFunction {
    id: Id<Function>,
    function: FunctionData,
    /// Types of inputs
    types: Vec<ValueType>,
    values: Vec<FieldElement>,
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
    function_signatures: HashMap<Id<Function>, FunctionSignature>,
    /// Stored functions
    stored_functions: Vec<StoredFunction>,
    /// Current function id
    current_function_id: Id<Function>,
}

impl FuzzerProgramContext {
    /// Creates a new FuzzerProgramContext
    pub(crate) fn new(program_context_options: FunctionContextOptions) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self {
            acir_builder,
            brillig_builder,
            program_context_options,
            is_constant: false,
            function_signatures: HashMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
        }
    }

    /// Creates a new FuzzerProgramContext where all inputs are constants
    pub(crate) fn new_constant_context(program_context_options: FunctionContextOptions) -> Self {
        let acir_builder = FuzzerBuilder::new_acir();
        let brillig_builder = FuzzerBuilder::new_brillig();
        Self {
            acir_builder,
            brillig_builder,
            program_context_options,
            is_constant: true,
            function_signatures: HashMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
        }
    }

    /// Stores function and its signature
    pub(crate) fn process_function(
        &mut self,
        function: FunctionData,
        types: Vec<ValueType>,
        values: Vec<impl Into<FieldElement> + Clone>,
    ) {
        let signature =
            FunctionSignature { input_types: types.clone(), return_type: function.return_type };
        self.function_signatures.insert(self.current_function_id, signature);
        let stored_function = StoredFunction {
            id: self.current_function_id,
            function,
            types,
            values: values.into_iter().map(|i| -> FieldElement { i.into() }).collect(),
        };
        self.stored_functions.push(stored_function);
        self.current_function_id = Id::new(self.current_function_id.to_u32() + 1);
    }

    /// Creates new ACIR and Brillig functions for each of stored functions and finalizes them
    pub(crate) fn finalize_program(&mut self) {
        for i in 0..self.stored_functions.len() {
            let stored_function = &self.stored_functions[i];
            // use only functions defined after this one to avoid recursion
            let defined_functions: HashMap<Id<Function>, FunctionSignature> = self
                .function_signatures
                .clone()
                .into_iter()
                .filter(|func_id| func_id.0.to_u32() > stored_function.id.to_u32())
                .collect();
            let mut function_context = if self.is_constant {
                FuzzerFunctionContext::new_constant_context(
                    (stored_function.values.to_vec(), stored_function.types.to_vec()),
                    &stored_function.function.blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type,
                    defined_functions,
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                )
            } else {
                FuzzerFunctionContext::new(
                    stored_function.types.to_vec(),
                    &stored_function.function.blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type,
                    defined_functions,
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                )
            };
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
