//! The `acir` module contains all the logic necessary for noirc's ACIR-gen pass which
//! generates the output ACIR program.
//!
//! # Usage
//!
//! ACIR generation is performed by calling the [Ssa::into_acir] method, providing any necessary brillig bytecode.
//! The compiled program will be returned as an [`Artifacts`] type.

use fxhash::FxHashMap as HashMap;
use noirc_errors::call_stack::CallStack;
use std::collections::{BTreeMap, HashSet};
use types::{AcirDynamicArray, AcirValue};

use acvm::acir::{
    BlackBoxFunc,
    circuit::{
        AssertionPayload, ExpressionWidth, OpcodeLocation, brillig::BrilligFunctionId,
        opcodes::AcirFunctionId,
    },
    native_types::Witness,
};
use acvm::{FieldElement, acir::AcirField, acir::circuit::opcodes::BlockId};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::monomorphization::ast::InlineType;

mod acir_context;
mod arrays;
pub(crate) mod ssa;
#[cfg(test)]
mod tests;
mod types;

use crate::brillig::BrilligOptions;
use crate::brillig::brillig_gen::gen_brillig_for;
use crate::brillig::{
    Brillig,
    brillig_gen::brillig_fn::FunctionContext as BrilligFunctionContext,
    brillig_ir::artifact::{BrilligParameter, GeneratedBrillig},
};
use crate::errors::{InternalError, InternalWarning, RuntimeError, SsaReport};
use crate::ssa::ir::instruction::Hint;
use crate::ssa::{
    function_builder::data_bus::DataBus,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, InstructionId, Intrinsic,
            TerminatorInstruction,
        },
        map::Id,
        printer::try_to_extract_string_from_error_payload,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

use acir_context::{AcirContext, BrilligStdLib, BrilligStdlibFunc, power_of_two};
use types::{AcirType, AcirVar};
pub use {acir_context::GeneratedAcir, ssa::Artifacts};

#[derive(Default)]
struct SharedContext<F: AcirField> {
    brillig_stdlib: BrilligStdLib<F>,

    /// Final list of Brillig functions which will be part of the final program
    /// This is shared across `Context` structs as we want one list of Brillig
    /// functions across all ACIR artifacts
    generated_brillig: Vec<GeneratedBrillig<F>>,

    /// Maps SSA function index -> Final generated Brillig artifact index.
    /// There can be Brillig functions specified in SSA which do not act as
    /// entry points in ACIR (e.g. only called by other Brillig functions)
    /// This mapping is necessary to use the correct function pointer for a Brillig call.
    /// This uses the brillig parameters in the map since using slices with different lengths
    /// needs to create different brillig entrypoints
    brillig_generated_func_pointers:
        BTreeMap<(FunctionId, Vec<BrilligParameter>), BrilligFunctionId>,

    /// Maps a Brillig std lib function (a handwritten primitive such as for inversion) -> Final generated Brillig artifact index.
    /// A separate mapping from normal Brillig calls is necessary as these methods do not have an associated function id from SSA.
    brillig_stdlib_func_pointer: HashMap<BrilligStdlibFunc, BrilligFunctionId>,

    /// Keeps track of Brillig std lib calls per function that need to still be resolved
    /// with the correct function pointer from the `brillig_stdlib_func_pointer` map.
    brillig_stdlib_calls_to_resolve: HashMap<FunctionId, Vec<(OpcodeLocation, BrilligFunctionId)>>,
}

impl<F: AcirField> SharedContext<F> {
    fn generated_brillig_pointer(
        &self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
    ) -> Option<&BrilligFunctionId> {
        self.brillig_generated_func_pointers.get(&(func_id, arguments))
    }

    fn generated_brillig(&self, func_pointer: usize) -> &GeneratedBrillig<F> {
        &self.generated_brillig[func_pointer]
    }

    fn insert_generated_brillig(
        &mut self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
        generated_pointer: BrilligFunctionId,
        code: GeneratedBrillig<F>,
    ) {
        self.brillig_generated_func_pointers.insert((func_id, arguments), generated_pointer);
        self.generated_brillig.push(code);
    }

    fn new_generated_pointer(&self) -> BrilligFunctionId {
        BrilligFunctionId(self.generated_brillig.len() as u32)
    }

    fn generate_brillig_calls_to_resolve(
        &mut self,
        brillig_stdlib_func: &BrilligStdlibFunc,
        func_id: FunctionId,
        opcode_location: OpcodeLocation,
    ) {
        if let Some(generated_pointer) =
            self.brillig_stdlib_func_pointer.get(brillig_stdlib_func).copied()
        {
            self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
        } else {
            let code = self.brillig_stdlib.get_code(*brillig_stdlib_func);
            let generated_pointer = self.new_generated_pointer();
            self.insert_generated_brillig_stdlib(
                *brillig_stdlib_func,
                generated_pointer,
                func_id,
                opcode_location,
                code.clone(),
            );
        }
    }

    /// Insert a newly generated Brillig stdlib function
    fn insert_generated_brillig_stdlib(
        &mut self,
        brillig_stdlib_func: BrilligStdlibFunc,
        generated_pointer: BrilligFunctionId,
        func_id: FunctionId,
        opcode_location: OpcodeLocation,
        code: GeneratedBrillig<F>,
    ) {
        self.brillig_stdlib_func_pointer.insert(brillig_stdlib_func, generated_pointer);
        self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
        self.generated_brillig.push(code);
    }

    fn add_call_to_resolve(
        &mut self,
        func_id: FunctionId,
        call_to_resolve: (OpcodeLocation, BrilligFunctionId),
    ) {
        self.brillig_stdlib_calls_to_resolve.entry(func_id).or_default().push(call_to_resolve);
    }
}

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context<'a> {
    /// Maps SSA values to `AcirVar`.
    ///
    /// This is needed so that we only create a single
    /// AcirVar per SSA value. Before creating an `AcirVar`
    /// for an SSA value, we check this map. If an `AcirVar`
    /// already exists for this Value, we return the `AcirVar`.
    ssa_values: HashMap<Id<Value>, AcirValue>,

    /// The `AcirVar` that describes the condition belonging to the most recently invoked
    /// `SideEffectsEnabled` instruction.
    current_side_effects_enabled_var: AcirVar,

    /// Manages and builds the `AcirVar`s to which the converted SSA values refer.
    acir_context: AcirContext<FieldElement, Bn254BlackBoxSolver>,

    /// Track initialized acir dynamic arrays
    ///
    /// An acir array must start with a MemoryInit ACIR opcodes
    /// and then have MemoryOp opcodes
    /// This set is used to ensure that a MemoryOp opcode is only pushed to the circuit
    /// if there is already a MemoryInit opcode.
    initialized_arrays: HashSet<BlockId>,

    /// Maps SSA values to BlockId
    /// A BlockId is an ACIR structure which identifies a memory block
    /// Each acir memory block corresponds to a different SSA array.
    memory_blocks: HashMap<Id<Value>, BlockId>,

    /// Maps SSA values to a BlockId used internally
    /// A BlockId is an ACIR structure which identifies a memory block
    /// Each memory blocks corresponds to a different SSA value
    /// which utilizes this internal memory for ACIR generation.
    internal_memory_blocks: HashMap<Id<Value>, BlockId>,

    /// Maps an internal memory block to its length
    ///
    /// This is necessary to keep track of an internal memory block's size.
    /// We do not need a separate map to keep track of `memory_blocks` as
    /// the length is set when we construct a `AcirValue::DynamicArray` and is tracked
    /// as part of the `AcirValue` in the `ssa_values` map.
    /// The length of an internal memory block is determined before an array operation
    /// takes place thus we track it separate here in this map.
    internal_mem_block_lengths: HashMap<BlockId, usize>,

    /// Number of the next BlockId, it is used to construct
    /// a new BlockId
    max_block_id: u32,

    data_bus: DataBus,

    /// Contains state that is generated and also used across ACIR functions
    shared_context: &'a mut SharedContext<FieldElement>,

    brillig: &'a Brillig,

    /// Options affecting Brillig code generation.
    brillig_options: &'a BrilligOptions,
}

impl<'a> Context<'a> {
    fn new(
        shared_context: &'a mut SharedContext<FieldElement>,
        expression_width: ExpressionWidth,
        brillig: &'a Brillig,
        brillig_stdlib: BrilligStdLib<FieldElement>,
        brillig_options: &'a BrilligOptions,
    ) -> Context<'a> {
        let mut acir_context = AcirContext::new(brillig_stdlib, Bn254BlackBoxSolver::default());
        acir_context.set_expression_width(expression_width);
        let current_side_effects_enabled_var = acir_context.add_constant(FieldElement::one());

        Context {
            ssa_values: HashMap::default(),
            current_side_effects_enabled_var,
            acir_context,
            initialized_arrays: HashSet::new(),
            memory_blocks: HashMap::default(),
            internal_memory_blocks: HashMap::default(),
            internal_mem_block_lengths: HashMap::default(),
            max_block_id: 0,
            data_bus: DataBus::default(),
            shared_context,
            brillig,
            brillig_options,
        }
    }

    fn convert_ssa_function(
        mut self,
        ssa: &Ssa,
        function: &Function,
    ) -> Result<Option<GeneratedAcir<FieldElement>>, RuntimeError> {
        self.acir_context.set_call_stack_helper(self.brillig.call_stacks.to_owned());
        match function.runtime() {
            RuntimeType::Acir(inline_type) => {
                match inline_type {
                    InlineType::Inline | InlineType::InlineAlways => {
                        if function.id() != ssa.main_id {
                            panic!(
                                "ACIR function should have been inlined earlier if not marked otherwise"
                            );
                        }
                    }
                    InlineType::NoPredicates => {
                        panic!(
                            "All ACIR functions marked with #[no_predicates] should be inlined before ACIR gen. This is an SSA exclusive codegen attribute"
                        );
                    }
                    InlineType::Fold => {}
                }
                // We only want to convert entry point functions. This being `main` and those marked with `InlineType::Fold`
                Ok(Some(self.convert_acir_main(function, ssa)?))
            }
            RuntimeType::Brillig(_) => {
                if function.id() == ssa.main_id {
                    Ok(Some(self.convert_brillig_main(function)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn convert_acir_main(
        mut self,
        main_func: &Function,
        ssa: &Ssa,
    ) -> Result<GeneratedAcir<FieldElement>, RuntimeError> {
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];
        let input_witness = self.convert_ssa_block_params(entry_block.parameters(), dfg)?;
        let num_return_witnesses =
            self.get_num_return_witnesses(entry_block.unwrap_terminator(), dfg);

        // Create a witness for each return witness we have to guarantee that the return witnesses match the standard
        // layout for serializing those types as if they were being passed as inputs.
        //
        // This is required for recursion as otherwise in situations where we cannot make use of the program's ABI
        // (e.g. for `std::verify_proof` or the solidity verifier), we need extra knowledge about the program we're
        // working with rather than following the standard ABI encoding rules.
        //
        // We allocate these witnesses now before performing ACIR gen for the rest of the program as the location of
        // the function's return values can then be determined through knowledge of its ABI alone.
        let return_witness_vars =
            vecmap(0..num_return_witnesses, |_| self.acir_context.add_variable());

        let return_witnesses = vecmap(&return_witness_vars, |return_var| {
            let expr = self.acir_context.var_to_expression(*return_var).unwrap();
            expr.to_witness().expect("return vars should be witnesses")
        });

        self.data_bus = dfg.data_bus.to_owned();
        let mut warnings = Vec::new();
        for instruction_id in entry_block.instructions() {
            warnings.extend(self.convert_ssa_instruction(*instruction_id, dfg, ssa)?);
        }
        let (return_vars, return_warnings) =
            self.convert_ssa_return(entry_block.unwrap_terminator(), dfg)?;

        // TODO: This is a naive method of assigning the return values to their witnesses as
        // we're likely to get a number of constraints which are asserting one witness to be equal to another.
        //
        // We should search through the program and relabel these witnesses so we can remove this constraint.
        for (witness_var, return_var) in return_witness_vars.iter().zip(return_vars) {
            self.acir_context.assert_eq_var(*witness_var, return_var, None)?;
        }

        self.initialize_databus(&return_witnesses, dfg)?;
        warnings.extend(return_warnings);
        warnings.extend(self.acir_context.warnings.clone());

        // Add the warnings from the alter Ssa passes
        Ok(self.acir_context.finish(
            input_witness,
            // Don't embed databus return witnesses into the circuit.
            if self.data_bus.return_data.is_some() { Vec::new() } else { return_witnesses },
            warnings,
        ))
    }

    fn convert_brillig_main(
        mut self,
        main_func: &Function,
    ) -> Result<GeneratedAcir<FieldElement>, RuntimeError> {
        let dfg = &main_func.dfg;

        let inputs = try_vecmap(dfg[main_func.entry_block()].parameters(), |param_id| {
            let typ = dfg.type_of_value(*param_id);
            self.create_value_from_type(&typ, &mut |this, _| Ok(this.acir_context.add_variable()))
        })?;
        let arguments = self.gen_brillig_parameters(dfg[main_func.entry_block()].parameters(), dfg);

        let witness_inputs = self.acir_context.extract_witness(&inputs);

        let outputs: Vec<AcirType> =
            vecmap(main_func.returns(), |result_id| dfg.type_of_value(*result_id).into());

        let code =
            gen_brillig_for(main_func, arguments.clone(), self.brillig, self.brillig_options)?;

        // We specifically do not attempt execution of the brillig code being generated as this can result in it being
        // replaced with constraints on witnesses to the program outputs.
        let output_values = self.acir_context.brillig_call(
            self.current_side_effects_enabled_var,
            &code,
            inputs,
            outputs,
            false,
            true,
            // We are guaranteed to have a Brillig function pointer of `0` as main itself is marked as unconstrained
            BrilligFunctionId(0),
            None,
        )?;
        self.shared_context.insert_generated_brillig(
            main_func.id(),
            arguments,
            BrilligFunctionId(0),
            code,
        );

        let return_witnesses: Vec<Witness> = output_values
            .iter()
            .flat_map(|value| value.clone().flatten())
            .map(|(value, _)| self.acir_context.var_to_witness(value))
            .collect::<Result<_, _>>()?;

        let generated_acir = self.acir_context.finish(witness_inputs, return_witnesses, Vec::new());

        assert_eq!(
            generated_acir.opcodes().len(),
            1,
            "Unconstrained programs should only generate a single opcode but multiple were emitted"
        );

        Ok(generated_acir)
    }

    /// Adds and binds `AcirVar`s for each numeric block parameter or block parameter array element.
    fn convert_ssa_block_params(
        &mut self,
        params: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<Witness>, RuntimeError> {
        // The first witness (if any) is the next one
        let start_witness = self.acir_context.current_witness_index().0;
        for param_id in params {
            let typ = dfg.type_of_value(*param_id);
            let value = self.convert_ssa_block_param(&typ)?;
            match &value {
                AcirValue::Var(_, _) => (),
                AcirValue::Array(_) => {
                    let block_id = self.block_id(param_id);
                    let len = if matches!(typ, Type::Array(_, _)) {
                        typ.flattened_size() as usize
                    } else {
                        return Err(InternalError::Unexpected {
                            expected: "Block params should be an array".to_owned(),
                            found: format!("Instead got {:?}", typ),
                            call_stack: self.acir_context.get_call_stack(),
                        }
                        .into());
                    };
                    self.initialize_array(block_id, len, Some(value.clone()))?;
                }
                AcirValue::DynamicArray(_) => unreachable!(
                    "The dynamic array type is created in Acir gen and therefore cannot be a block parameter"
                ),
            }
            self.ssa_values.insert(*param_id, value);
        }
        let end_witness = self.acir_context.current_witness_index().0;
        let witnesses = (start_witness..=end_witness).map(Witness::from).collect();
        Ok(witnesses)
    }

    fn convert_ssa_block_param(&mut self, param_type: &Type) -> Result<AcirValue, RuntimeError> {
        self.create_value_from_type(param_type, &mut |this, typ| this.add_numeric_input_var(&typ))
    }

    fn create_value_from_type(
        &mut self,
        param_type: &Type,
        make_var: &mut impl FnMut(&mut Self, NumericType) -> Result<AcirVar, RuntimeError>,
    ) -> Result<AcirValue, RuntimeError> {
        match param_type {
            Type::Numeric(numeric_type) => {
                let typ = AcirType::new(*numeric_type);
                Ok(AcirValue::Var(make_var(self, *numeric_type)?, typ))
            }
            Type::Array(element_types, length) => {
                let mut elements = im::Vector::new();

                for _ in 0..*length {
                    for element in element_types.iter() {
                        elements.push_back(self.create_value_from_type(element, make_var)?);
                    }
                }

                Ok(AcirValue::Array(elements))
            }
            _ => unreachable!("ICE: Params to the program should only contains numbers and arrays"),
        }
    }

    /// Creates an `AcirVar` corresponding to a parameter witness to appears in the abi. A range
    /// constraint is added if the numeric type requires it.
    ///
    /// This function is used not only for adding numeric block parameters, but also for adding
    /// any array elements that belong to reference type block parameters.
    fn add_numeric_input_var(
        &mut self,
        numeric_type: &NumericType,
    ) -> Result<AcirVar, RuntimeError> {
        let acir_var = self.acir_context.add_variable();
        let one = self.acir_context.add_constant(FieldElement::one());
        if matches!(numeric_type, NumericType::Signed { .. } | NumericType::Unsigned { .. }) {
            // The predicate is one so that this constraint is is always applied.
            self.acir_context.range_constrain_var(acir_var, numeric_type, None, one)?;
        }
        Ok(acir_var)
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
    ) -> Result<Vec<SsaReport>, RuntimeError> {
        let instruction = &dfg[instruction_id];
        self.acir_context.set_call_stack(dfg.get_instruction_call_stack(instruction_id));
        let mut warnings = Vec::new();
        // Disable the side effects if the binary instruction does not require them
        let one = self.acir_context.add_constant(FieldElement::one());
        let predicate = if instruction.requires_acir_gen_predicate(dfg) {
            self.current_side_effects_enabled_var
        } else {
            one
        };

        match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self.convert_ssa_binary(binary, dfg, predicate)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let lhs = self.convert_numeric_value(*lhs, dfg)?;
                let rhs = self.convert_numeric_value(*rhs, dfg)?;

                let assert_payload = if let Some(error) = assert_message {
                    match error {
                        ConstrainError::StaticString(string) => Some(
                            self.acir_context.generate_assertion_message_payload(string.clone()),
                        ),
                        ConstrainError::Dynamic(error_selector, is_string_type, values) => {
                            if let Some(constant_string) = try_to_extract_string_from_error_payload(
                                *is_string_type,
                                values,
                                dfg,
                            ) {
                                Some(
                                    self.acir_context
                                        .generate_assertion_message_payload(constant_string),
                                )
                            } else {
                                let acir_vars: Vec<_> = values
                                    .iter()
                                    .map(|value| self.convert_value(*value, dfg))
                                    .collect();

                                let expressions_or_memory =
                                    self.acir_context.vars_to_expressions_or_memory(&acir_vars)?;

                                Some(AssertionPayload {
                                    error_selector: error_selector.as_u64(),
                                    payload: expressions_or_memory,
                                })
                            }
                        }
                    }
                } else {
                    None
                };

                self.acir_context.assert_eq_var(lhs, rhs, assert_payload)?;
            }
            Instruction::ConstrainNotEqual(lhs, rhs, assert_message) => {
                let lhs = self.convert_numeric_value(*lhs, dfg)?;
                let rhs = self.convert_numeric_value(*rhs, dfg)?;

                let assert_payload = if let Some(error) = assert_message {
                    match error {
                        ConstrainError::StaticString(string) => Some(
                            self.acir_context.generate_assertion_message_payload(string.clone()),
                        ),
                        ConstrainError::Dynamic(error_selector, is_string_type, values) => {
                            if let Some(constant_string) = try_to_extract_string_from_error_payload(
                                *is_string_type,
                                values,
                                dfg,
                            ) {
                                Some(
                                    self.acir_context
                                        .generate_assertion_message_payload(constant_string),
                                )
                            } else {
                                let acir_vars: Vec<_> = values
                                    .iter()
                                    .map(|value| self.convert_value(*value, dfg))
                                    .collect();

                                let expressions_or_memory =
                                    self.acir_context.vars_to_expressions_or_memory(&acir_vars)?;

                                Some(AssertionPayload {
                                    error_selector: error_selector.as_u64(),
                                    payload: expressions_or_memory,
                                })
                            }
                        }
                    }
                } else {
                    None
                };

                self.acir_context.assert_neq_var(
                    lhs,
                    rhs,
                    self.current_side_effects_enabled_var,
                    assert_payload,
                )?;
            }
            Instruction::Cast(value_id, _) => {
                let acir_var = self.convert_numeric_value(*value_id, dfg)?;
                self.define_result_var(dfg, instruction_id, acir_var);
            }
            Instruction::Call { .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                warnings.extend(self.convert_ssa_call(instruction, dfg, ssa, result_ids)?);
            }
            Instruction::Not(value_id) => {
                let (acir_var, typ) = match self.convert_value(*value_id, dfg) {
                    AcirValue::Var(acir_var, typ) => (acir_var, typ),
                    _ => unreachable!("NOT is only applied to numerics"),
                };
                let result_acir_var = self.acir_context.not_var(acir_var, typ)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                let result_acir_var =
                    self.convert_ssa_truncate(*value, *bit_size, *max_bit_size, dfg)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::EnableSideEffectsIf { condition } => {
                let acir_var = self.convert_numeric_value(*condition, dfg)?;
                self.current_side_effects_enabled_var = acir_var;
            }
            Instruction::ArrayGet { .. } | Instruction::ArraySet { .. } => {
                self.handle_array_operation(instruction_id, dfg)?;
            }
            Instruction::Allocate => {
                return Err(RuntimeError::UnknownReference {
                    call_stack: self.acir_context.get_call_stack().clone(),
                });
            }
            Instruction::Store { .. } => {
                unreachable!("Expected all store instructions to be removed before acir_gen")
            }
            Instruction::Load { .. } => {
                unreachable!("Expected all load instructions to be removed before acir_gen")
            }
            Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => {
                // Only Brillig needs to worry about reference counted arrays
                unreachable!("Expected all Rc instructions to be removed before acir_gen")
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                let acir_var = self.convert_numeric_value(*value, dfg)?;
                // Predicate is one because the predicate has already been
                // handled in the RangeCheck instruction during the flattening pass.
                self.acir_context.range_constrain_var(
                    acir_var,
                    &NumericType::Unsigned { bit_size: *max_bit_size },
                    assert_message.clone(),
                    one,
                )?;
            }
            Instruction::IfElse { .. } => {
                unreachable!("IfElse instruction remaining in acir-gen")
            }
            Instruction::MakeArray { elements, typ: _ } => {
                let elements = elements.iter().map(|element| self.convert_value(*element, dfg));
                let value = AcirValue::Array(elements.collect());
                let result = dfg.instruction_results(instruction_id)[0];
                self.ssa_values.insert(result, value);
            }
            Instruction::Noop => (),
        }

        self.acir_context.set_call_stack(CallStack::new());
        Ok(warnings)
    }

    fn convert_ssa_call(
        &mut self,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        result_ids: &[ValueId],
    ) -> Result<Vec<SsaReport>, RuntimeError> {
        let mut warnings = Vec::new();

        match instruction {
            Instruction::Call { func, arguments } => {
                let function_value = &dfg[*func];
                match function_value {
                    Value::Function(id) => {
                        let func = &ssa.functions[id];
                        match func.runtime() {
                            RuntimeType::Acir(inline_type) => {
                                assert!(
                                    !matches!(inline_type, InlineType::Inline),
                                    "ICE: Got an ACIR function named {} that should have already been inlined",
                                    func.name()
                                );

                                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
                                let output_count = result_ids
                                    .iter()
                                    .map(|result_id| {
                                        dfg.type_of_value(*result_id).flattened_size() as usize
                                    })
                                    .sum();

                                let Some(acir_function_id) = ssa.get_entry_point_index(id) else {
                                    unreachable!(
                                        "Expected an associated final index for call to acir function {id} with args {arguments:?}"
                                    );
                                };

                                let output_vars = self.acir_context.call_acir_function(
                                    AcirFunctionId(acir_function_id),
                                    inputs,
                                    output_count,
                                    self.current_side_effects_enabled_var,
                                )?;

                                let output_values =
                                    self.convert_vars_to_values(output_vars, dfg, result_ids);

                                self.handle_ssa_call_outputs(result_ids, output_values, dfg)?;
                            }
                            RuntimeType::Brillig(_) => {
                                // Check that we are not attempting to return a slice from
                                // an unconstrained runtime to a constrained runtime
                                for result_id in result_ids {
                                    if dfg.type_of_value(*result_id).contains_slice_element() {
                                        return Err(
                                            RuntimeError::UnconstrainedSliceReturnToConstrained {
                                                call_stack: self.acir_context.get_call_stack(),
                                            },
                                        );
                                    }
                                }
                                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
                                let arguments = self.gen_brillig_parameters(arguments, dfg);

                                let outputs: Vec<AcirType> = vecmap(result_ids, |result_id| {
                                    dfg.type_of_value(*result_id).into()
                                });

                                // Check whether we have already generated Brillig for this function
                                // If we have, re-use the generated code to set-up the Brillig call.
                                let output_values = if let Some(generated_pointer) = self
                                    .shared_context
                                    .generated_brillig_pointer(*id, arguments.clone())
                                {
                                    let code = self
                                        .shared_context
                                        .generated_brillig(generated_pointer.as_usize());
                                    self.acir_context.brillig_call(
                                        self.current_side_effects_enabled_var,
                                        code,
                                        inputs,
                                        outputs,
                                        true,
                                        false,
                                        *generated_pointer,
                                        None,
                                    )?
                                } else {
                                    let code = gen_brillig_for(
                                        func,
                                        arguments.clone(),
                                        self.brillig,
                                        self.brillig_options,
                                    )?;
                                    let generated_pointer =
                                        self.shared_context.new_generated_pointer();
                                    let output_values = self.acir_context.brillig_call(
                                        self.current_side_effects_enabled_var,
                                        &code,
                                        inputs,
                                        outputs,
                                        true,
                                        false,
                                        generated_pointer,
                                        None,
                                    )?;
                                    self.shared_context.insert_generated_brillig(
                                        *id,
                                        arguments,
                                        generated_pointer,
                                        code,
                                    );
                                    output_values
                                };

                                // Compiler sanity check
                                assert_eq!(
                                    result_ids.len(),
                                    output_values.len(),
                                    "ICE: The number of Brillig output values should match the result ids in SSA"
                                );

                                self.handle_ssa_call_outputs(result_ids, output_values, dfg)?;
                            }
                        }
                    }
                    Value::Intrinsic(intrinsic) => {
                        if matches!(
                            intrinsic,
                            Intrinsic::BlackBox(BlackBoxFunc::RecursiveAggregation)
                        ) {
                            warnings.push(SsaReport::Warning(InternalWarning::VerifyProof {
                                call_stack: self.acir_context.get_call_stack(),
                            }));
                        }
                        let outputs = self
                            .convert_ssa_intrinsic_call(*intrinsic, arguments, dfg, result_ids)?;

                        // Issue #1438 causes this check to fail with intrinsics that return 0
                        // results but the ssa form instead creates 1 unit result value.
                        // assert_eq!(result_ids.len(), outputs.len());
                        self.handle_ssa_call_outputs(result_ids, outputs, dfg)?;
                    }
                    Value::ForeignFunction(_) => {
                        // TODO: Remove this once elaborator is default frontend. This is now caught by a lint inside the frontend.
                        return Err(RuntimeError::UnconstrainedOracleReturnToConstrained {
                            call_stack: self.acir_context.get_call_stack(),
                        });
                    }
                    _ => unreachable!("expected calling a function but got {function_value:?}"),
                }
            }
            _ => unreachable!("expected calling a call instruction"),
        }
        Ok(warnings)
    }

    fn handle_ssa_call_outputs(
        &mut self,
        result_ids: &[ValueId],
        output_values: Vec<AcirValue>,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        for (result_id, output) in result_ids.iter().zip(output_values) {
            if let AcirValue::Array(_) = &output {
                let array_id = *result_id;
                let block_id = self.block_id(&array_id);
                let array_typ = dfg.type_of_value(array_id);
                let len = if matches!(array_typ, Type::Array(_, _)) {
                    array_typ.flattened_size() as usize
                } else {
                    arrays::flattened_value_size(&output)
                };
                self.initialize_array(block_id, len, Some(output.clone()))?;
            }
            // Do nothing for AcirValue::DynamicArray and AcirValue::Var
            // A dynamic array returned from a function call should already be initialized
            // and a single variable does not require any extra initialization.
            self.ssa_values.insert(*result_id, output);
        }
        Ok(())
    }

    fn gen_brillig_parameters(
        &self,
        values: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Vec<BrilligParameter> {
        values
            .iter()
            .map(|&value_id| {
                let typ = dfg.type_of_value(value_id);
                if let Type::Slice(item_types) = typ {
                    let len = match self
                        .ssa_values
                        .get(&value_id)
                        .expect("ICE: Unknown slice input to brillig")
                    {
                        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => *len,
                        AcirValue::Array(array) => array.len(),
                        _ => unreachable!("ICE: Slice value is not an array"),
                    };

                    BrilligParameter::Slice(
                        item_types
                            .iter()
                            .map(BrilligFunctionContext::ssa_type_to_parameter)
                            .collect(),
                        len / item_types.len(),
                    )
                } else {
                    BrilligFunctionContext::ssa_type_to_parameter(&typ)
                }
            })
            .collect()
    }

    /// Remember the result of an instruction returning a single value
    fn define_result(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirValue,
    ) {
        let result_ids = dfg.instruction_results(instruction);
        self.ssa_values.insert(result_ids[0], result);
    }

    /// Remember the result of instruction returning a single numeric value
    fn define_result_var(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirVar,
    ) {
        let result_ids = dfg.instruction_results(instruction);
        let typ = dfg.type_of_value(result_ids[0]).into();
        self.define_result(dfg, instruction, AcirValue::Var(result, typ));
    }

    /// Converts an SSA terminator's return values into their ACIR representations
    fn get_num_return_witnesses(
        &self,
        terminator: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) -> usize {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values, .. } => return_values,
            TerminatorInstruction::Unreachable { .. } => return 0,
            // TODO(https://github.com/noir-lang/noir/issues/4616): Enable recursion on foldable/non-inlined ACIR functions
            TerminatorInstruction::JmpIf { .. } | TerminatorInstruction::Jmp { .. } => {
                unreachable!("ICE: Program must have a singular return")
            }
        };

        return_values
            .iter()
            .fold(0, |acc, value_id| acc + dfg.type_of_value(*value_id).flattened_size() as usize)
    }

    /// Converts an SSA terminator's return values into their ACIR representations
    fn convert_ssa_return(
        &mut self,
        terminator: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) -> Result<(Vec<AcirVar>, Vec<SsaReport>), RuntimeError> {
        let (return_values, call_stack) = match terminator {
            TerminatorInstruction::Return { return_values, call_stack } => {
                (return_values, *call_stack)
            }
            // TODO(https://github.com/noir-lang/noir/issues/4616): Enable recursion on foldable/non-inlined ACIR functions
            TerminatorInstruction::JmpIf { .. } | TerminatorInstruction::Jmp { .. } => {
                unreachable!("ICE: Program must have a singular return")
            }
            TerminatorInstruction::Unreachable { .. } => return Ok((vec![], vec![])),
        };

        let mut has_constant_return = false;
        let mut return_vars: Vec<AcirVar> = Vec::new();
        for value_id in return_values {
            let value = self.convert_value(*value_id, dfg);

            // `value` may or may not be an array reference. Calling `flatten` will expand the array if there is one.
            let acir_vars = self.acir_context.flatten(value)?;
            for (acir_var, _) in acir_vars {
                has_constant_return |= self.acir_context.is_constant(&acir_var);
                return_vars.push(acir_var);
            }
        }

        let call_stack = dfg.call_stack_data.get_call_stack(call_stack);
        let warnings = if has_constant_return {
            vec![SsaReport::Warning(InternalWarning::ReturnConstant { call_stack })]
        } else {
            Vec::new()
        };

        Ok((return_vars, warnings))
    }

    /// Gets the cached `AcirVar` that was converted from the corresponding `ValueId`. If it does
    /// not already exist in the cache, a conversion is attempted and cached for simple values
    /// that require no further context such as numeric types - values requiring more context
    /// should have already been cached elsewhere.
    ///
    /// Conversion is assumed to have already been performed for instruction results and block
    /// parameters. This is because block parameters are converted before anything else, and
    /// because instructions results are converted when the corresponding instruction is
    /// encountered. (An instruction result cannot be referenced before the instruction occurs.)
    ///
    /// It is not safe to call this function on value ids that represent addresses. Instructions
    /// involving such values are evaluated via a separate path and stored in
    /// `ssa_value_to_array_address` instead.
    fn convert_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirValue {
        let value = &dfg[value_id];
        if let Some(acir_value) = self.ssa_values.get(&value_id) {
            return acir_value.clone();
        }

        let acir_value = match value {
            Value::NumericConstant { constant, typ } => {
                let typ = AcirType::from(Type::Numeric(*typ));
                AcirValue::Var(self.acir_context.add_constant(*constant), typ)
            }
            Value::Intrinsic(..) => todo!(),
            Value::Function(function_id) => {
                // This conversion is for debugging support only, to allow the
                // debugging instrumentation code to work. Taking the reference
                // of a function in ACIR is useless.
                let id = self.acir_context.add_constant(function_id.to_u32());
                AcirValue::Var(id, AcirType::field())
            }
            Value::ForeignFunction(_) => unimplemented!(
                "Oracle calls directly in constrained functions are not yet available."
            ),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value_id} {value:?}")
            }
            Value::Global(_) => {
                unreachable!("ICE: All globals should have been inlined");
            }
        };
        self.ssa_values.insert(value_id, acir_value.clone());
        acir_value
    }

    fn convert_numeric_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, InternalError> {
        match self.convert_value(value_id, dfg) {
            AcirValue::Var(acir_var, _) => Ok(acir_var),
            AcirValue::Array(array) => Err(InternalError::Unexpected {
                expected: "a numeric value".to_string(),
                found: format!("{array:?}"),
                call_stack: self.acir_context.get_call_stack(),
            }),
            AcirValue::DynamicArray(_) => Err(InternalError::Unexpected {
                expected: "a numeric value".to_string(),
                found: "an array".to_string(),
                call_stack: self.acir_context.get_call_stack(),
            }),
        }
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        predicate: AcirVar,
    ) -> Result<AcirVar, RuntimeError> {
        let lhs = self.convert_numeric_value(binary.lhs, dfg)?;
        let rhs = self.convert_numeric_value(binary.rhs, dfg)?;
        let binary_type = self.type_of_binary_operation(binary, dfg);

        if binary_type.is_signed()
            && matches!(
                binary.operator,
                BinaryOp::Add { unchecked: false }
                    | BinaryOp::Sub { unchecked: false }
                    | BinaryOp::Mul { unchecked: false }
            )
        {
            panic!("Checked signed operations should all be removed before ACIRgen")
        }

        let binary_type = AcirType::from(binary_type);
        let bit_count = binary_type.bit_size::<FieldElement>();
        let num_type = binary_type.to_numeric_type();
        let result = match binary.operator {
            BinaryOp::Add { .. } => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub { .. } => self.acir_context.sub_var(lhs, rhs),
            BinaryOp::Mul { .. } => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(lhs, rhs, binary_type.clone(), predicate),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => match binary_type {
                AcirType::NumericType(NumericType::Signed { .. }) => {
                    self.acir_context.less_than_signed(lhs, rhs, bit_count)
                }
                _ => self.acir_context.less_than_var(lhs, rhs, bit_count),
            },
            BinaryOp::Xor => self.acir_context.xor_var(lhs, rhs, binary_type),
            BinaryOp::And => self.acir_context.and_var(lhs, rhs, binary_type),
            BinaryOp::Or => self.acir_context.or_var(lhs, rhs, binary_type),
            BinaryOp::Mod => {
                self.acir_context.modulo_var(lhs, rhs, binary_type.clone(), bit_count, predicate)
            }
            BinaryOp::Shl | BinaryOp::Shr => unreachable!(
                "ICE - bit shift operators do not exist in ACIR and should have been replaced"
            ),
        }?;

        if let NumericType::Unsigned { bit_size } = &num_type {
            // Check for integer overflow
            self.check_unsigned_overflow(result, *bit_size, binary, predicate)
        } else {
            Ok(result)
        }
    }

    /// Adds a range check against the bit size of the result of addition, subtraction or multiplication
    fn check_unsigned_overflow(
        &mut self,
        result: AcirVar,
        bit_size: u32,
        binary: &Binary,
        predicate: AcirVar,
    ) -> Result<AcirVar, RuntimeError> {
        let msg = match binary.operator {
            BinaryOp::Add { unchecked: false } => "attempt to add with overflow",
            BinaryOp::Sub { unchecked: false } => "attempt to subtract with overflow",
            BinaryOp::Mul { unchecked: false } => "attempt to multiply with overflow",
            _ => return Ok(result),
        };

        self.acir_context.range_constrain_var(
            result,
            &NumericType::Unsigned { bit_size },
            Some(msg.to_string()),
            predicate,
        )
    }

    /// Operands in a binary operation are checked to have the same type.
    ///
    /// In Noir, binary operands should have the same type due to the language
    /// semantics.
    ///
    /// There are some edge cases to consider:
    /// - Constants are not explicitly type casted, so we need to check for this and
    ///   return the type of the other operand, if we have a constant.
    /// - 0 is not seen as `Field 0` but instead as `Unit 0`
    ///
    /// TODO: The latter seems like a bug, if we cannot differentiate between a function returning
    /// TODO nothing and a 0.
    ///
    /// TODO: This constant coercion should ideally be done in the type checker.
    fn type_of_binary_operation(&self, binary: &Binary, dfg: &DataFlowGraph) -> Type {
        let lhs_type = dfg.type_of_value(binary.lhs);
        let rhs_type = dfg.type_of_value(binary.rhs);

        match (lhs_type, rhs_type) {
            // Function type should not be possible, since all functions
            // have been inlined.
            (_, Type::Function) | (Type::Function, _) => {
                unreachable!("all functions should be inlined")
            }
            (_, Type::Reference(_)) | (Type::Reference(_), _) => {
                unreachable!("References are invalid in binary operations")
            }
            (_, Type::Array(..)) | (Type::Array(..), _) => {
                unreachable!("Arrays are invalid in binary operations")
            }
            (_, Type::Slice(..)) | (Type::Slice(..), _) => {
                unreachable!("Arrays are invalid in binary operations")
            }
            // If either side is a Field constant then, we coerce into the type
            // of the other operand
            (Type::Numeric(NumericType::NativeField), typ)
            | (typ, Type::Numeric(NumericType::NativeField)) => typ,
            // If either side is a numeric type, then we expect their types to be
            // the same.
            (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
                assert_eq!(lhs_type, rhs_type, "lhs and rhs types in {binary:?} are not the same");
                Type::Numeric(lhs_type)
            }
        }
    }

    /// Returns an `AcirVar`that is constrained to be result of the truncation.
    fn convert_ssa_truncate(
        &mut self,
        value_id: ValueId,
        bit_size: u32,
        mut max_bit_size: u32,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        assert_ne!(bit_size, max_bit_size, "Attempted to generate a noop truncation");
        assert!(
            bit_size < max_bit_size,
            "Attempted to generate a truncation into size larger than max input"
        );

        let mut var = self.convert_numeric_value(value_id, dfg)?;
        match &dfg[value_id] {
            Value::Instruction { instruction, .. } => {
                if matches!(
                    &dfg[*instruction],
                    Instruction::Binary(Binary { operator: BinaryOp::Sub { .. }, .. })
                ) {
                    // Subtractions must first have the integer modulus added before truncation can be
                    // applied. This is done in order to prevent underflow.
                    //
                    // FieldElements have max bit size equals to max_num_bits so
                    // we filter out this bit size because there is no underflow
                    // for FieldElements. Furthermore, adding a power of two
                    // would be incorrect for a FieldElement (cf. #8519).
                    if max_bit_size < FieldElement::max_num_bits() {
                        let integer_modulus = power_of_two::<FieldElement>(max_bit_size);
                        let integer_modulus = self.acir_context.add_constant(integer_modulus);
                        var = self.acir_context.add_var(var, integer_modulus)?;
                        max_bit_size += 1;
                    }
                }
            }
            Value::Param { .. } => {
                // Binary operations on params may have been entirely simplified if the operation
                // results in the identity of the parameter
            }
            _ => unreachable!(
                "ICE: Truncates are only ever applied to the result of a binary op or a param"
            ),
        };

        self.acir_context.truncate_var(var, bit_size, max_bit_size)
    }

    /// Returns a vector of `AcirVar`s constrained to be result of the function call.
    ///
    /// The function being called is required to be intrinsic.
    fn convert_ssa_intrinsic_call(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        match intrinsic {
            Intrinsic::Hint(Hint::BlackBox) => {
                // Identity function; at the ACIR level this is a no-op, it only affects the SSA.
                assert_eq!(
                    arguments.len(),
                    result_ids.len(),
                    "ICE: BlackBox input and output lengths should match."
                );
                Ok(arguments.iter().map(|v| self.convert_value(*v, dfg)).collect())
            }
            Intrinsic::BlackBox(black_box) => {
                // Slices are represented as a tuple of (length, slice contents).
                // We must check the inputs to determine if there are slices
                // and make sure that we pass the correct inputs to the black box function call.
                // The loop below only keeps the slice contents, so that
                // setting up a black box function with slice inputs matches the expected
                // number of arguments specified in the function signature.
                let mut arguments_no_slice_len = Vec::new();
                for (i, arg) in arguments.iter().enumerate() {
                    if matches!(dfg.type_of_value(*arg), Type::Numeric(_)) {
                        if i < arguments.len() - 1 {
                            if !matches!(dfg.type_of_value(arguments[i + 1]), Type::Slice(_)) {
                                arguments_no_slice_len.push(*arg);
                            }
                        } else {
                            arguments_no_slice_len.push(*arg);
                        }
                    } else {
                        arguments_no_slice_len.push(*arg);
                    }
                }

                let inputs = vecmap(&arguments_no_slice_len, |arg| self.convert_value(*arg, dfg));

                let output_count = result_ids.iter().fold(0usize, |sum, result_id| {
                    sum + dfg.type_of_value(*result_id).flattened_size() as usize
                });

                let vars = self.acir_context.black_box_function(black_box, inputs, output_count)?;

                Ok(self.convert_vars_to_values(vars, dfg, result_ids))
            }
            Intrinsic::ApplyRangeConstraint => {
                unreachable!(
                    "ICE: `Intrinsic::ApplyRangeConstraint` calls should be transformed into an `Instruction::RangeCheck`"
                );
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let radix = self.convert_value(arguments[1], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToRadix result must be an array");
                };

                self.acir_context
                    .radix_decompose(
                        endian,
                        field,
                        radix,
                        array_length,
                        result_type[0].clone().into(),
                    )
                    .map(|array| vec![array])
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToBits result must be an array");
                };

                self.acir_context
                    .bit_decompose(endian, field, array_length, result_type[0].clone().into())
                    .map(|array| vec![array])
            }
            Intrinsic::ArrayLen => {
                let len = match self.convert_value(arguments[0], dfg) {
                    AcirValue::Var(_, _) => unreachable!("Non-array passed to array.len() method"),
                    AcirValue::Array(values) => values.len(),
                    AcirValue::DynamicArray(array) => array.len,
                };
                Ok(vec![AcirValue::Var(self.acir_context.add_constant(len), AcirType::field())])
            }
            Intrinsic::AsSlice => {
                let slice_contents = arguments[0];
                let slice_typ = dfg.type_of_value(slice_contents);
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice_length = self.flattened_size(slice_contents, dfg);
                let slice_length = self.acir_context.add_constant(slice_length);

                let acir_value = self.convert_value(slice_contents, dfg);
                let result = self.read_array(acir_value)?;

                Ok(vec![AcirValue::Var(slice_length, AcirType::field()), AcirValue::Array(result)])
            }
            Intrinsic::SlicePushBack => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];
                let elements_to_push = &arguments[2..];

                let slice_typ = dfg.type_of_value(slice_contents);

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                // Increase the slice length by one to enable accessing more elements in the slice.
                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let slice = self.convert_value(slice_contents, dfg);
                let mut new_slice = self.read_array(slice)?;

                // We must directly push back elements for non-nested slices
                for elem in elements_to_push {
                    let element = self.convert_value(*elem, dfg);
                    new_slice.push_back(element);
                }

                let new_slice_val = AcirValue::Array(new_slice);
                let new_elem_size = arrays::flattened_value_size(&new_slice_val);
                let value_types = new_slice_val.clone().flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    new_elem_size,
                    "ICE: Value types array must match new slice size"
                );

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
            }
            Intrinsic::SlicePushFront => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];
                let elements_to_push = &arguments[2..];
                let slice_typ = dfg.type_of_value(slice_contents);
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                // Increase the slice length by one to enable accessing more elements in the slice.
                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let slice = self.convert_value(slice_contents, dfg);
                let mut new_slice = self.read_array(slice)?;

                // We must directly push front elements for non-nested slices
                for elem in elements_to_push.iter().rev() {
                    let element = self.convert_value(*elem, dfg);
                    new_slice.push_front(element);
                }

                let new_slice_val = AcirValue::Array(new_slice);
                let new_slice_size = arrays::flattened_value_size(&new_slice_val);

                let value_types = new_slice_val.clone().flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    new_slice_size,
                    "ICE: Value types array must match new slice size"
                );

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
            }
            Intrinsic::SlicePopBack => {
                // arguments = [slice_length, slice_contents]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;
                // For a pop back operation we want to fetch from the `length - 1` as this is the
                // last valid index that can be accessed in a slice. After the pop back operation
                // the elements stored at that index will no longer be able to be accessed.
                let mut var_index = new_slice_length;

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let mut popped_elements = Vec::new();
                for res in &result_ids[2..] {
                    let elem =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
                    popped_elements.push(elem);
                }

                let slice = self.convert_value(slice_contents, dfg);
                let new_slice = self.read_array(slice)?;

                let mut results = vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ];
                results.append(&mut popped_elements);

                Ok(results)
            }
            Intrinsic::SlicePopFront => {
                // arguments = [slice_length, slice_contents]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let slice = self.convert_value(slice_contents, dfg);

                let mut new_slice = self.read_array(slice)?;

                let element_size = slice_typ.element_size();

                let mut popped_elements: Vec<AcirValue> = Vec::new();
                let mut popped_elements_size = 0;
                let mut var_index = self.acir_context.add_constant(FieldElement::zero());
                // Fetch the values we are popping off of the slice.
                // In the case of non-nested slice the logic is simple as we do not
                // need to account for the internal slice sizes or flattening the index.
                for res in &result_ids[..element_size] {
                    let element =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
                    let elem_size = arrays::flattened_value_size(&element);
                    popped_elements_size += elem_size;
                    popped_elements.push(element);
                }

                // It is expected that the `popped_elements_size` is the flattened size of the elements,
                // as the input slice should be a dynamic array which is represented by flat memory.
                new_slice = new_slice.slice(popped_elements_size..);

                popped_elements.push(AcirValue::Var(new_slice_length, AcirType::field()));
                popped_elements.push(AcirValue::Array(new_slice));

                Ok(popped_elements)
            }
            Intrinsic::SliceInsert => {
                // arguments = [slice_length, slice_contents, insert_index, ...elements_to_insert]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let insert_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let slice_size = arrays::flattened_value_size(&slice);

                // Fetch the flattened index from the user provided index argument.
                let element_size = slice_typ.element_size();
                let element_size_var = self.acir_context.add_constant(element_size);
                let flat_insert_index =
                    self.acir_context.mul_var(insert_index, element_size_var)?;
                let flat_user_index =
                    self.get_flattened_index(&slice_typ, slice_contents, flat_insert_index, dfg)?;

                let elements_to_insert = &arguments[3..];
                // Determine the elements we need to write into our resulting dynamic array.
                // We need to a fully flat list of AcirVar's as a dynamic array is represented with flat memory.
                let mut inner_elem_size_usize = 0;
                let mut flattened_elements = Vec::new();
                for elem in elements_to_insert {
                    let element = self.convert_value(*elem, dfg);
                    let elem_size = arrays::flattened_value_size(&element);
                    inner_elem_size_usize += elem_size;
                    let mut flat_elem = element.flatten().into_iter().map(|(var, _)| var).collect();
                    flattened_elements.append(&mut flat_elem);
                }
                let inner_elem_size = self.acir_context.add_constant(inner_elem_size_usize);
                // Set the maximum flattened index at which a new element should be inserted.
                let max_flat_user_index =
                    self.acir_context.add_var(flat_user_index, inner_elem_size)?;

                // Go through the entire slice argument and determine what value should be written to the new slice.
                // 1. If we are below the starting insertion index we should insert the value that was already
                //    in the original slice.
                // 2. If we are above the starting insertion index but below the max insertion index we should insert
                //    the flattened element arguments.
                // 3. If we are above the max insertion index we should insert the previous value from the original slice,
                //    as during an insertion we want to shift all elements after the insertion up an index.
                let result_block_id = self.block_id(&result_ids[1]);
                self.initialize_array(result_block_id, slice_size, None)?;
                let mut current_insert_index = 0;
                for i in 0..slice_size {
                    let current_index = self.acir_context.add_constant(i);

                    // Check that we are above the lower bound of the insertion index
                    let greater_eq_than_idx =
                        self.acir_context.more_than_eq_var(current_index, flat_user_index, 64)?;
                    // Check that we are below the upper bound of the insertion index
                    let less_than_idx =
                        self.acir_context.less_than_var(current_index, max_flat_user_index, 64)?;

                    // Read from the original slice the value we want to insert into our new slice.
                    // We need to make sure that we read the previous element when our current index is greater than insertion index.
                    // If the index for the previous element is out of the array bounds we can avoid the check for whether
                    // the current index is over the insertion index.
                    let shifted_index = if i < inner_elem_size_usize {
                        current_index
                    } else {
                        let index_minus_elem_size =
                            self.acir_context.add_constant(i - inner_elem_size_usize);

                        let use_shifted_index_pred = self
                            .acir_context
                            .mul_var(index_minus_elem_size, greater_eq_than_idx)?;

                        let not_pred = self.acir_context.sub_var(one, greater_eq_than_idx)?;
                        let use_current_index_pred =
                            self.acir_context.mul_var(not_pred, current_index)?;

                        self.acir_context.add_var(use_shifted_index_pred, use_current_index_pred)?
                    };

                    let value_shifted_index =
                        self.acir_context.read_from_memory(block_id, &shifted_index)?;

                    // Final predicate to determine whether we are within the insertion bounds
                    let should_insert_value_pred =
                        self.acir_context.mul_var(greater_eq_than_idx, less_than_idx)?;
                    let insert_value_pred = self.acir_context.mul_var(
                        flattened_elements[current_insert_index],
                        should_insert_value_pred,
                    )?;

                    let not_pred = self.acir_context.sub_var(one, should_insert_value_pred)?;
                    let shifted_value_pred =
                        self.acir_context.mul_var(not_pred, value_shifted_index)?;

                    let new_value =
                        self.acir_context.add_var(insert_value_pred, shifted_value_pred)?;

                    self.acir_context.write_to_memory(
                        result_block_id,
                        &current_index,
                        &new_value,
                    )?;

                    current_insert_index += 1;
                    if inner_elem_size_usize == current_insert_index {
                        current_insert_index = 0;
                    }
                }

                let element_type_sizes =
                    if arrays::array_has_constant_element_size(&slice_typ).is_none() {
                        Some(self.init_element_type_sizes_array(
                            &slice_typ,
                            slice_contents,
                            Some(&slice),
                            dfg,
                        )?)
                    } else {
                        None
                    };

                let value_types = slice.flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    slice_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: slice_size,
                    value_types,
                    element_type_sizes,
                });

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), result])
            }
            Intrinsic::SliceRemove => {
                // arguments = [slice_length, slice_contents, remove_index]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let remove_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let slice_size = arrays::flattened_value_size(&slice);

                let new_slice = self.read_array(slice)?;

                // Compiler sanity check
                assert_eq!(
                    new_slice.len(),
                    slice_size,
                    "ICE: The read flattened slice should match the computed size"
                );

                // Fetch the flattened index from the user provided index argument.
                let element_size = slice_typ.element_size();
                let element_size_var = self.acir_context.add_constant(element_size);
                let flat_remove_index =
                    self.acir_context.mul_var(remove_index, element_size_var)?;
                let flat_user_index =
                    self.get_flattened_index(&slice_typ, slice_contents, flat_remove_index, dfg)?;

                // Fetch the values we are remove from the slice.
                // As we fetch the values we can determine the size of the removed values
                // which we will later use for writing the correct resulting slice.
                let mut popped_elements = Vec::new();
                let mut popped_elements_size = 0;
                // Set a temp index just for fetching from the original slice as `array_get_value` mutates
                // the index internally.
                let mut temp_index = flat_user_index;
                for res in &result_ids[2..(2 + element_size)] {
                    let element =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut temp_index)?;
                    let elem_size = arrays::flattened_value_size(&element);
                    popped_elements_size += elem_size;
                    popped_elements.push(element);
                }

                // Go through the entire slice argument and determine what value should be written to the new slice.
                // 1. If the current index is greater than the removal index we must write the next value
                //    from the original slice to the current index
                // 2. At the end of the slice reading from the next value of the original slice
                //    can lead to a potential out of bounds error. In this case we just fetch from the original slice
                //    at the current index. As we are decreasing the slice in length, this is a safe operation.
                let result_block_id = self.block_id(&result_ids[1]);
                self.initialize_array(
                    result_block_id,
                    slice_size,
                    Some(AcirValue::Array(new_slice.clone())),
                )?;
                for i in 0..slice_size {
                    let current_index = self.acir_context.add_constant(i);

                    let value_current_index = &new_slice[i].borrow_var()?;

                    if slice_size > (i + popped_elements_size) {
                        let shifted_index =
                            self.acir_context.add_constant(i + popped_elements_size);

                        let value_shifted_index =
                            self.acir_context.read_from_memory(block_id, &shifted_index)?;

                        let use_shifted_value = self.acir_context.more_than_eq_var(
                            current_index,
                            flat_user_index,
                            64,
                        )?;

                        let shifted_value_pred =
                            self.acir_context.mul_var(value_shifted_index, use_shifted_value)?;
                        let not_pred = self.acir_context.sub_var(one, use_shifted_value)?;
                        let current_value_pred =
                            self.acir_context.mul_var(not_pred, *value_current_index)?;

                        let new_value =
                            self.acir_context.add_var(shifted_value_pred, current_value_pred)?;

                        self.acir_context.write_to_memory(
                            result_block_id,
                            &current_index,
                            &new_value,
                        )?;
                    };
                }

                let new_slice_val = AcirValue::Array(new_slice);
                let element_type_sizes =
                    if arrays::array_has_constant_element_size(&slice_typ).is_none() {
                        Some(self.init_element_type_sizes_array(
                            &slice_typ,
                            slice_contents,
                            Some(&new_slice_val),
                            dfg,
                        )?)
                    } else {
                        None
                    };

                let value_types = new_slice_val.flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    slice_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: slice_size,
                    value_types,
                    element_type_sizes,
                });

                let mut result = vec![AcirValue::Var(new_slice_length, AcirType::field()), result];
                result.append(&mut popped_elements);

                Ok(result)
            }

            Intrinsic::AsWitness => {
                let arg = arguments[0];
                let input = self.convert_value(arg, dfg).into_var()?;
                Ok(self
                    .acir_context
                    .get_or_create_witness_var(input)
                    .map(|val| self.convert_vars_to_values(vec![val], dfg, result_ids))?)
            }
            Intrinsic::ArrayAsStrUnchecked => Ok(vec![self.convert_value(arguments[0], dfg)]),
            Intrinsic::AssertConstant => {
                unreachable!("Expected assert_constant to be removed by this point")
            }
            Intrinsic::StaticAssert => {
                unreachable!("Expected static_assert to be removed by this point")
            }
            Intrinsic::StrAsBytes => unreachable!("Expected as_bytes to be removed by this point"),
            Intrinsic::IsUnconstrained => {
                unreachable!("Expected is_unconstrained to be removed by this point")
            }
            Intrinsic::DerivePedersenGenerators => {
                unreachable!("DerivePedersenGenerators can only be called with constants")
            }
            Intrinsic::FieldLessThan => {
                unreachable!("FieldLessThan can only be called in unconstrained")
            }
            Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                let zero = self.acir_context.add_constant(FieldElement::zero());
                Ok(vec![AcirValue::Var(
                    zero,
                    AcirType::NumericType(NumericType::Unsigned { bit_size: 32 }),
                )])
            }
        }
    }

    /// Convert a `Vec<AcirVar>` into a `Vec<AcirValue>` using the given result ids.
    /// If the type of a result id is an array, several acir vars are collected into
    /// a single AcirValue::Array of the same length.
    /// If the type of a result id is a slice, the slice length must precede it and we can
    /// convert to an AcirValue::Array when the length is known (constant).
    fn convert_vars_to_values(
        &self,
        vars: Vec<AcirVar>,
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Vec<AcirValue> {
        let mut vars = vars.into_iter();
        let mut values: Vec<AcirValue> = Vec::new();
        for result in result_ids {
            let result_type = dfg.type_of_value(*result);
            if let Type::Slice(elements_type) = result_type {
                let error = "ICE - cannot get slice length when converting slice to AcirValue";
                let len = values.last().expect(error).borrow_var().expect(error);
                let len = self.acir_context.constant(len).to_u128();
                let mut element_values = im::Vector::new();
                for _ in 0..len {
                    for element_type in elements_type.iter() {
                        let element = Self::convert_var_type_to_values(element_type, &mut vars);
                        element_values.push_back(element);
                    }
                }
                values.push(AcirValue::Array(element_values));
            } else {
                values.push(Self::convert_var_type_to_values(&result_type, &mut vars));
            }
        }
        values
    }

    /// Recursive helper for convert_vars_to_values.
    /// If the given result_type is an array of length N, this will create an AcirValue::Array with
    /// the first N elements of the given iterator. Otherwise, the result is a single
    /// AcirValue::Var wrapping the first element of the iterator.
    fn convert_var_type_to_values(
        result_type: &Type,
        vars: &mut impl Iterator<Item = AcirVar>,
    ) -> AcirValue {
        match result_type {
            Type::Array(elements, size) => {
                let mut element_values = im::Vector::new();
                for _ in 0..*size {
                    for element_type in elements.iter() {
                        let element = Self::convert_var_type_to_values(element_type, vars);
                        element_values.push_back(element);
                    }
                }
                AcirValue::Array(element_values)
            }
            typ => {
                let var = vars.next().unwrap();
                AcirValue::Var(var, typ.into())
            }
        }
    }
}
