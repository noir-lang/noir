//! The `acir` module contains all the logic necessary for noirc's ACIR-gen pass which
//! generates the output ACIR program.
//!
//! # Usage
//!
//! ACIR generation is performed by calling the [Ssa::into_acir] method, providing any necessary brillig bytecode.
//! The compiled program will be returned as an [`Artifacts`] type.

use noirc_artifacts::ssa::{InternalWarning, SsaReport};
use noirc_errors::call_stack::CallStack;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use types::{AcirDynamicArray, AcirValue};

use acvm::acir::{
    circuit::{AssertionPayload, brillig::BrilligFunctionId},
    native_types::Witness,
};
use acvm::{FieldElement, acir::AcirField, acir::circuit::opcodes::BlockId};
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::monomorphization::ast::InlineType;

mod acir_context;
mod arrays;
mod call;
mod shared_context;
pub(crate) mod ssa;
#[cfg(test)]
mod tests;
mod types;

use crate::brillig::Brillig;
use crate::brillig::brillig_gen::gen_brillig_for;
use crate::errors::{InternalError, RuntimeError};
use crate::ssa::{
    function_builder::data_bus::DataBus,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, InstructionId, TerminatorInstruction,
        },
        map::Id,
        printer::try_to_extract_string_from_error_payload,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use crate::{acir::shared_context::SharedContext, brillig::BrilligOptions};

use acir_context::{AcirContext, BrilligStdLib, power_of_two};
use types::{AcirType, AcirVar};
pub use {acir_context::GeneratedAcir, ssa::Artifacts};

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
struct Context<'a> {
    /// Maps SSA values to `AcirVar`'s.
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
    acir_context: AcirContext<FieldElement>,

    /// Track initialized acir dynamic arrays
    ///
    /// An acir array must start with a MemoryInit ACIR opcodes
    /// and then have MemoryOp opcodes
    /// This set is used to ensure that a MemoryOp opcode is only pushed to the circuit
    /// if there is already a MemoryInit opcode.
    initialized_arrays: HashSet<BlockId>,

    /// Maps SSA values to BlockId's
    /// A BlockId is an ACIR structure which identifies a memory block
    /// Each acir memory block corresponds to a different SSA array.
    memory_blocks: HashMap<Id<Value>, BlockId>,

    /// The BlockId dedicated to return_data
    /// It is not managed by memory_blocks to ensure getting always a fresh block for return_data, even if
    /// the SSA array has already been initialized to a block.
    return_data_block_id: Option<BlockId>,

    /// Maps SSA values to BlockId's used internally for computing the accurate flattened
    /// index of non-homogenous arrays.
    /// See [arrays] for more information about the purpose of the type sizes array.
    ///
    /// A BlockId is an ACIR structure which identifies a memory block
    /// Each memory blocks corresponds to a different SSA value
    /// which utilizes this internal memory for ACIR generation.
    element_type_sizes_blocks: HashMap<Id<Value>, BlockId>,

    /// Maps type sizes to BlockId. This is used to reuse the same BlockId if different
    /// non-homogenous arrays end up having the same type sizes layout.
    type_sizes_to_blocks: HashMap<Vec<u32>, BlockId>,

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
        brillig: &'a Brillig,
        brillig_stdlib: BrilligStdLib<FieldElement>,
        brillig_options: &'a BrilligOptions,
    ) -> Context<'a> {
        let mut acir_context = AcirContext::new(brillig_stdlib);
        let current_side_effects_enabled_var = acir_context.add_constant(FieldElement::one());

        Context {
            ssa_values: HashMap::default(),
            current_side_effects_enabled_var,
            acir_context,
            initialized_arrays: HashSet::default(),
            memory_blocks: HashMap::default(),
            return_data_block_id: None,
            element_type_sizes_blocks: HashMap::default(),
            type_sizes_to_blocks: HashMap::default(),
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
        self.acir_context.set_call_stack_helper(self.brillig.call_stacks().clone());
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
                    InlineType::InlineNever => {
                        panic!(
                            "ACIR function marked with #[inline_never]. This attribute is only allowed on unconstrained functions"
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
        self.acir_context.acir_ir.input_witnesses =
            self.convert_ssa_block_params(entry_block.parameters(), dfg)?;

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

        let mut warnings = Vec::new();

        let used_globals = self.shared_context.get_and_remove_used_globals_set(main_func.id());

        let globals_dfg = (*main_func.dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals_dfg);
        for (id, value) in globals_dfg.values_iter() {
            if !used_globals.contains(&id) {
                continue;
            }
            match value {
                Value::NumericConstant { .. } => {
                    self.convert_value(id, dfg);
                }
                Value::Instruction { instruction, .. } => {
                    warnings.extend(self.convert_ssa_instruction(
                        *instruction,
                        &globals_dfg,
                        ssa,
                    )?);
                }
                _ => {
                    panic!(
                        "Expected either an instruction or a numeric constant for a global value"
                    )
                }
            }
        }

        self.data_bus = dfg.data_bus.to_owned();
        for instruction_id in entry_block.instructions() {
            warnings.extend(self.convert_ssa_instruction(*instruction_id, dfg, ssa)?);
        }
        let (return_vars, return_warnings) =
            self.convert_ssa_return(entry_block.unwrap_terminator(), dfg)?;

        // This is a naive method of assigning the return values to their witnesses as
        // we're likely to get a number of constraints which are asserting one witness to be equal to another.
        //
        // But an attempt at searching through the program and relabeling these witnesses so we could remove
        // this constraint was [closed](https://github.com/noir-lang/noir/pull/10112#event-20171150226)
        // but "the opcode count doesn't even change in real circuits."
        for (witness_var, return_var) in return_witness_vars.iter().zip(return_vars) {
            self.acir_context.assert_eq_var(*witness_var, return_var, None)?;
        }

        self.initialize_databus(&return_witnesses, dfg)?;
        warnings.extend(return_warnings);
        warnings.extend(self.acir_context.warnings.clone());

        #[cfg(debug_assertions)]
        acir_post_check(&self, &self.acir_context.acir_ir);

        // Add the warnings from the alter Ssa passes
        Ok(self.acir_context.finish(
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

        self.acir_context.acir_ir.input_witnesses = self.acir_context.extract_witnesses(&inputs);
        let returns = main_func.returns().unwrap_or_default();

        let outputs: Vec<AcirType> =
            vecmap(returns, |result_id| dfg.type_of_value(*result_id).into());

        let code =
            gen_brillig_for(main_func, arguments.clone(), self.brillig, self.brillig_options)?;

        // We specifically do not attempt execution of the brillig code being generated as this can result in it being
        // replaced with constraints on witnesses to the program outputs.
        let unsafe_return_values = true;
        let output_values = self.acir_context.brillig_call(
            self.current_side_effects_enabled_var,
            &code,
            inputs,
            outputs,
            unsafe_return_values,
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

        let generated_acir = self.acir_context.finish(return_witnesses, Vec::new());

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
        for &param_id in params {
            let typ = dfg.type_of_value(param_id);
            let value = self.convert_ssa_block_param(&typ)?;
            match &value {
                AcirValue::Var(_, _) => (),
                AcirValue::Array(_) => {
                    let block_id = self.block_id(param_id);
                    let len = if matches!(typ, Type::Array(_, _)) {
                        typ.flattened_size()
                    } else {
                        return Err(InternalError::Unexpected {
                            expected: "Block params should be an array".to_owned(),
                            found: format!("Instead got {typ:?}"),
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
            self.ssa_values.insert(param_id, value);
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
                Ok(AcirValue::Var(make_var(self, *numeric_type)?, *numeric_type))
            }
            Type::Array(element_types, length) => {
                let mut elements = im::Vector::new();

                for _ in 0..length.0 {
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

        if !numeric_type.is_field() {
            let one = self.acir_context.add_constant(FieldElement::one());
            // The predicate is one so that this constraint is is always applied to Signed/Unsigned NumericTypes

            self.acir_context.range_constrain_var(
                acir_var,
                numeric_type.bit_size::<FieldElement>(),
                None,
                one,
            )?;
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

        match instruction {
            Instruction::Binary(binary) => {
                // Disable the side effects if the binary instruction does not require them
                let predicate = if instruction.requires_acir_gen_predicate(dfg) {
                    self.current_side_effects_enabled_var
                } else {
                    self.acir_context.add_constant(FieldElement::one())
                };
                let result_acir_var = self.convert_ssa_binary(binary, dfg, predicate)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let lhs = self.convert_numeric_value(*lhs, dfg)?;
                let rhs = self.convert_numeric_value(*rhs, dfg)?;
                let assert_payload = self.convert_constrain_error(dfg, assert_message)?;
                self.acir_context.assert_eq_var(lhs, rhs, assert_payload)?;
            }
            Instruction::ConstrainNotEqual(lhs, rhs, assert_message) => {
                let lhs = self.convert_numeric_value(*lhs, dfg)?;
                let rhs = self.convert_numeric_value(*rhs, dfg)?;
                let assert_payload = self.convert_constrain_error(dfg, assert_message)?;
                let predicate = self.current_side_effects_enabled_var;
                self.acir_context.assert_neq_var(lhs, rhs, predicate, assert_payload)?;
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
                let one = self.acir_context.add_constant(FieldElement::one());
                // Predicate is one because the predicate has already been
                // handled in the RangeCheck instruction during the flattening pass.
                self.acir_context.range_constrain_var(
                    acir_var,
                    *max_bit_size,
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
                let [result] = dfg.instruction_result(instruction_id);
                self.ssa_values.insert(result, value);
            }
            Instruction::Noop => (),
        }

        self.acir_context.set_call_stack(CallStack::new());
        Ok(warnings)
    }

    /// Converts an optional constrain error message into an ACIR assertion payload
    fn convert_constrain_error(
        &mut self,
        dfg: &DataFlowGraph,
        assert_message: &Option<ConstrainError>,
    ) -> Result<Option<AssertionPayload<FieldElement>>, RuntimeError> {
        let Some(error) = assert_message else {
            return Ok(None);
        };

        let assert_payload = match error {
            ConstrainError::StaticString(string) => {
                self.acir_context.generate_assertion_message_payload(string.clone())
            }
            ConstrainError::Dynamic(error_selector, is_string_type, values) => {
                if let Some(constant_string) =
                    try_to_extract_string_from_error_payload(*is_string_type, values, dfg)
                {
                    self.acir_context.generate_assertion_message_payload(constant_string)
                } else {
                    let acir_values: Vec<_> =
                        vecmap(values, |value| self.convert_value(*value, dfg));

                    let expressions_or_memory =
                        self.acir_context.values_to_expressions_or_memory(&acir_values)?;

                    let error_selector = error_selector.as_u64();
                    AssertionPayload { error_selector, payload: expressions_or_memory }
                }
            }
        };
        Ok(Some(assert_payload))
    }

    /// Remember the result of an instruction returning a single value
    fn define_result(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirValue,
    ) {
        let [result_id] = dfg.instruction_result(instruction);
        self.ssa_values.insert(result_id, result);
    }

    /// Remember the result of instruction returning a single numeric value
    fn define_result_var(
        &mut self,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
        result: AcirVar,
    ) {
        let [result_id] = dfg.instruction_result(instruction);
        let typ = dfg.type_of_value(result_id).unwrap_numeric();
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
            .fold(0, |acc, value_id| acc + dfg.type_of_value(*value_id).flattened_size().to_usize())
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
    /// It is not safe to call this function on value ids that represent pointers. Instructions
    /// involving such values are evaluated via a separate path and stored in
    /// `ssa_value_to_array_address` instead.
    fn convert_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirValue {
        assert!(
            !matches!(dfg.type_of_value(value_id), Type::Reference(_)),
            "convert_value: did not expect a Reference type"
        );

        let value = &dfg[value_id];
        if let Some(acir_value) = self.ssa_values.get(&value_id) {
            return acir_value.clone();
        }

        let acir_value = match value {
            Value::NumericConstant { constant, typ } => {
                AcirValue::Var(self.acir_context.add_constant(*constant), *typ)
            }
            Value::Intrinsic(..) => {
                unreachable!("ICE: Intrinsics should be resolved via separate logic")
            }
            Value::Function(function_id) => {
                // This conversion is for debugging support only, to allow the
                // debugging instrumentation code to work. Taking the reference
                // of a function in ACIR is useless.
                let id = self.acir_context.add_constant(function_id.to_u32());
                AcirValue::Var(id, NumericType::NativeField)
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
        let num_type = self.type_of_binary_operation(binary, dfg).unwrap_numeric();

        if num_type.is_signed()
            && matches!(
                binary.operator,
                BinaryOp::Add { unchecked: false }
                    | BinaryOp::Sub { unchecked: false }
                    | BinaryOp::Mul { unchecked: false }
            )
        {
            panic!("Checked signed operations should all be removed before ACIRgen")
        }

        let result = match binary.operator {
            BinaryOp::Add { .. } => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub { .. } => self.acir_context.sub_var(lhs, rhs),
            BinaryOp::Mul { .. } => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(lhs, rhs, num_type, predicate),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => match num_type {
                NumericType::Unsigned { bit_size } => {
                    self.acir_context.less_than_var(lhs, rhs, bit_size)
                }
                _ => {
                    panic!("ICE: unexpected binary type for Lt operation: {num_type:?}")
                }
            },
            BinaryOp::Xor => self.acir_context.xor_var(lhs, rhs, num_type),
            BinaryOp::And => self.acir_context.and_var(lhs, rhs, num_type),
            BinaryOp::Or => self.acir_context.or_var(lhs, rhs, num_type),
            BinaryOp::Mod => match num_type {
                NumericType::Unsigned { bit_size } => {
                    self.acir_context.modulo_var(lhs, rhs, bit_size, predicate)
                }
                _ => {
                    panic!("ICE: unexpected binary type for Mod operation: {num_type:?}")
                }
            },
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

        self.acir_context.range_constrain_var(result, bit_size, Some(msg.to_string()), predicate)
    }

    /// Operands in a binary operation are checked to have the same type.
    ///
    /// In Noir, binary operands should have the same type due to the language
    /// semantics.
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
            (_, Type::Vector(..)) | (Type::Vector(..), _) => {
                unreachable!("Arrays are invalid in binary operations")
            }
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
                        // When max_bit_size is max_num_bits() - 1, adding
                        // 2**max_bit_size to an element of max_bit_size bits
                        // gives an element of max_num_bits() bits which may overflow
                        assert!(
                            max_bit_size != FieldElement::max_num_bits() - 1,
                            "potential underflow in subtraction when max_bit_size is {max_bit_size}"
                        );
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

    /// Fetch a flat list of [AcirVar].
    ///
    /// Flattens an [AcirValue] into a vector of `AcirVar`.
    ///
    /// This is an extension of [AcirValue::flatten] that also supports [AcirValue::DynamicArray].
    fn flatten(&mut self, value: &AcirValue) -> Result<Vec<AcirVar>, RuntimeError> {
        Ok(match value {
            AcirValue::Var(var, _) => vec![*var],
            AcirValue::Array(array) => {
                let mut result = Vec::new();
                for elem in array {
                    result.extend(self.flatten(elem)?);
                }
                result
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, value_types, .. }) => {
                let elements = self.read_dynamic_array(*block_id, *len, value_types);
                let mut result = Vec::new();

                for value in elements {
                    match value? {
                        AcirValue::Var(var, _typ) => result.push(var),
                        _ => unreachable!("ICE: Dynamic memory should already be flat"),
                    }
                }
                result
            }
        })
    }
}

/// Check post ACIR generation properties
/// * No memory opcodes should be laid down that write to the internal type sizes array.
///   See [arrays] for more information on the type sizes array.
#[cfg(debug_assertions)]
fn acir_post_check(context: &Context<'_>, acir: &GeneratedAcir<FieldElement>) {
    use acvm::acir::circuit::Opcode;
    for opcode in acir.opcodes() {
        let Opcode::MemoryOp { block_id, op } = opcode else {
            continue;
        };
        if op.operation.is_one() {
            // Check that we have no writes to the type size arrays
            let is_type_sizes_array =
                context.element_type_sizes_blocks.values().any(|id| id == block_id);
            assert!(
                !is_type_sizes_array,
                "ICE: Writes to the internal type sizes array are forbidden"
            );
        }
    }
}
