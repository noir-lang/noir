//! This file holds the pass to convert from Noir's SSA IR to ACIR.
mod acir_ir;

use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;

use self::acir_ir::acir_variable::{AcirContext, AcirType, AcirVar};
use self::acir_ir::generated_acir::BrilligStdlibFunc;
use super::function_builder::data_bus::DataBus;
use super::ir::dfg::CallStack;
use super::ir::function::FunctionId;
use super::ir::instruction::{ConstrainError, ErrorType};
use super::ir::printer::try_to_extract_string_from_error_payload;
use super::{
    ir::{
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
        },
        map::Id,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use crate::brillig::brillig_ir::artifact::{BrilligParameter, GeneratedBrillig};
use crate::brillig::brillig_ir::BrilligContext;
use crate::brillig::{brillig_gen::brillig_fn::FunctionContext as BrilligFunctionContext, Brillig};
use crate::errors::{InternalError, InternalWarning, RuntimeError, SsaReport};
pub(crate) use acir_ir::generated_acir::GeneratedAcir;
use acvm::acir::circuit::opcodes::BlockType;
use noirc_frontend::monomorphization::ast::InlineType;

use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::acir::circuit::{AssertionPayload, ErrorSelector, OpcodeLocation};
use acvm::acir::native_types::Witness;
use acvm::acir::BlackBoxFunc;
use acvm::{
    acir::{circuit::opcodes::BlockId, native_types::Expression},
    FieldElement,
};
use fxhash::FxHashMap as HashMap;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};

#[derive(Default)]
struct SharedContext {
    /// Final list of Brillig functions which will be part of the final program
    /// This is shared across `Context` structs as we want one list of Brillig
    /// functions across all ACIR artifacts
    generated_brillig: Vec<GeneratedBrillig>,

    /// Maps SSA function index -> Final generated Brillig artifact index.
    /// There can be Brillig functions specified in SSA which do not act as
    /// entry points in ACIR (e.g. only called by other Brillig functions)
    /// This mapping is necessary to use the correct function pointer for a Brillig call.
    /// This uses the brillig parameters in the map since using slices with different lengths
    /// needs to create different brillig entrypoints
    brillig_generated_func_pointers: BTreeMap<(FunctionId, Vec<BrilligParameter>), u32>,

    /// Maps a Brillig std lib function (a handwritten primitive such as for inversion) -> Final generated Brillig artifact index.
    /// A separate mapping from normal Brillig calls is necessary as these methods do not have an associated function id from SSA.
    brillig_stdlib_func_pointer: HashMap<BrilligStdlibFunc, u32>,

    /// Keeps track of Brillig std lib calls per function that need to still be resolved
    /// with the correct function pointer from the `brillig_stdlib_func_pointer` map.
    brillig_stdlib_calls_to_resolve: HashMap<FunctionId, Vec<(OpcodeLocation, u32)>>,
}

impl SharedContext {
    fn generated_brillig_pointer(
        &self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
    ) -> Option<&u32> {
        self.brillig_generated_func_pointers.get(&(func_id, arguments))
    }

    fn generated_brillig(&self, func_pointer: usize) -> &GeneratedBrillig {
        &self.generated_brillig[func_pointer]
    }

    fn insert_generated_brillig(
        &mut self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
        generated_pointer: u32,
        code: GeneratedBrillig,
    ) {
        self.brillig_generated_func_pointers.insert((func_id, arguments), generated_pointer);
        self.generated_brillig.push(code);
    }

    fn new_generated_pointer(&self) -> u32 {
        self.generated_brillig.len() as u32
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
            let code = brillig_stdlib_func.get_generated_brillig();
            let generated_pointer = self.new_generated_pointer();
            self.insert_generated_brillig_stdlib(
                *brillig_stdlib_func,
                generated_pointer,
                func_id,
                opcode_location,
                code,
            );
        }
    }

    /// Insert a newly generated Brillig stdlib function
    fn insert_generated_brillig_stdlib(
        &mut self,
        brillig_stdlib_func: BrilligStdlibFunc,
        generated_pointer: u32,
        func_id: FunctionId,
        opcode_location: OpcodeLocation,
        code: GeneratedBrillig,
    ) {
        self.brillig_stdlib_func_pointer.insert(brillig_stdlib_func, generated_pointer);
        self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
        self.generated_brillig.push(code);
    }

    fn add_call_to_resolve(&mut self, func_id: FunctionId, call_to_resolve: (OpcodeLocation, u32)) {
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
    acir_context: AcirContext,

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
    shared_context: &'a mut SharedContext,
}

#[derive(Clone)]
pub(crate) struct AcirDynamicArray {
    /// Identification for the Acir dynamic array
    /// This is essentially a ACIR pointer to the array
    block_id: BlockId,
    /// Length of the array
    len: usize,
    /// An ACIR dynamic array is a flat structure, so we use
    /// the inner structure of an `AcirType::NumericType` directly.
    /// Some usages of ACIR arrays (e.g. black box functions) require the bit size
    /// of every value to be known, thus we store the types as part of the dynamic
    /// array definition.
    ///
    /// A dynamic non-homogenous array can potentially have values of differing types.
    /// Thus, we store a vector of types rather than a single type, as a dynamic non-homogenous array
    /// is still represented in ACIR by a single `AcirDynamicArray` structure.
    ///
    /// The length of the value types vector must match the `len` field in this structure.
    value_types: Vec<NumericType>,
    /// Identification for the ACIR dynamic array
    /// inner element type sizes array
    element_type_sizes: Option<BlockId>,
}
impl Debug for AcirDynamicArray {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "id: {}, len: {}, element_type_sizes: {:?}",
            self.block_id.0,
            self.len,
            self.element_type_sizes.map(|block_id| block_id.0)
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AcirValue {
    Var(AcirVar, AcirType),
    Array(im::Vector<AcirValue>),
    DynamicArray(AcirDynamicArray),
}

impl AcirValue {
    fn into_var(self) -> Result<AcirVar, InternalError> {
        match self {
            AcirValue::Var(var, _) => Ok(var),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => Err(InternalError::General {
                message: "Called AcirValue::into_var on an array".to_string(),
                call_stack: CallStack::new(),
            }),
        }
    }

    fn borrow_var(&self) -> Result<AcirVar, InternalError> {
        match self {
            AcirValue::Var(var, _) => Ok(*var),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => Err(InternalError::General {
                message: "Called AcirValue::borrow_var on an array".to_string(),
                call_stack: CallStack::new(),
            }),
        }
    }

    fn flatten(self) -> Vec<(AcirVar, AcirType)> {
        match self {
            AcirValue::Var(var, typ) => vec![(var, typ)],
            AcirValue::Array(array) => array.into_iter().flat_map(AcirValue::flatten).collect(),
            AcirValue::DynamicArray(_) => unimplemented!("Cannot flatten a dynamic array"),
        }
    }

    fn flat_numeric_types(self) -> Vec<NumericType> {
        match self {
            AcirValue::Array(_) => {
                self.flatten().into_iter().map(|(_, typ)| typ.to_numeric_type()).collect()
            }
            AcirValue::DynamicArray(AcirDynamicArray { value_types, .. }) => value_types,
            _ => unreachable!("An AcirValue::Var cannot be used as an array value"),
        }
    }
}

pub(crate) type Artifacts =
    (Vec<GeneratedAcir>, Vec<BrilligBytecode>, BTreeMap<ErrorSelector, ErrorType>);

impl Ssa {
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn into_acir(self, brillig: &Brillig) -> Result<Artifacts, RuntimeError> {
        let mut acirs = Vec::new();
        // TODO: can we parallelise this?
        let mut shared_context = SharedContext::default();
        for function in self.functions.values() {
            let context = Context::new(&mut shared_context);
            if let Some(mut generated_acir) =
                context.convert_ssa_function(&self, function, brillig)?
            {
                // We want to be able to insert Brillig stdlib functions anywhere during the ACIR generation process (e.g. such as on the `GeneratedAcir`).
                // As we don't want a reference to the `SharedContext` on the generated ACIR itself,
                // we instead store the opcode location at which a Brillig call to a std lib function occurred.
                // We then defer resolving the function IDs of those Brillig functions to when we have generated Brillig
                // for all normal Brillig calls.
                for (opcode_location, brillig_stdlib_func) in
                    &generated_acir.brillig_stdlib_func_locations
                {
                    shared_context.generate_brillig_calls_to_resolve(
                        brillig_stdlib_func,
                        function.id(),
                        *opcode_location,
                    );
                }

                // Fetch the Brillig stdlib calls to resolve for this function
                if let Some(calls_to_resolve) =
                    shared_context.brillig_stdlib_calls_to_resolve.get(&function.id())
                {
                    // Resolve the Brillig stdlib calls
                    // We have to do a separate loop as the generated ACIR cannot be borrowed as mutable after an immutable borrow
                    for (opcode_location, brillig_function_pointer) in calls_to_resolve {
                        generated_acir.resolve_brillig_stdlib_call(
                            *opcode_location,
                            *brillig_function_pointer,
                        );
                    }
                }

                generated_acir.name = function.name().to_owned();
                acirs.push(generated_acir);
            }
        }

        let brillig = vecmap(shared_context.generated_brillig, |brillig| BrilligBytecode {
            bytecode: brillig.byte_code,
        });

        let runtime_types = self.functions.values().map(|function| function.runtime());
        for (acir, runtime_type) in acirs.iter_mut().zip(runtime_types) {
            if matches!(runtime_type, RuntimeType::Acir(_)) {
                generate_distinct_return_witnesses(acir);
            }
        }

        Ok((acirs, brillig, self.error_selector_to_type))
    }
}

fn generate_distinct_return_witnesses(acir: &mut GeneratedAcir) {
    // Create a witness for each return witness we have to guarantee that the return witnesses match the standard
    // layout for serializing those types as if they were being passed as inputs.
    //
    // This is required for recursion as otherwise in situations where we cannot make use of the program's ABI
    // (e.g. for `std::verify_proof` or the solidity verifier), we need extra knowledge about the program we're
    // working with rather than following the standard ABI encoding rules.
    //
    // TODO: We're being conservative here by generating a new witness for every expression.
    // This means that we're likely to get a number of constraints which are just renumbering witnesses.
    // This can be tackled by:
    // - Tracking the last assigned public input witness and only renumbering a witness if it is below this value.
    // - Modifying existing constraints to rearrange their outputs so they are suitable
    //   - See: https://github.com/noir-lang/noir/pull/4467
    let distinct_return_witness = vecmap(acir.return_witnesses.clone(), |return_witness| {
        acir.create_witness_for_expression(&Expression::from(return_witness))
    });

    acir.return_witnesses = distinct_return_witness;
}

impl<'a> Context<'a> {
    fn new(shared_context: &'a mut SharedContext) -> Context<'a> {
        let mut acir_context = AcirContext::default();
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
        }
    }

    fn convert_ssa_function(
        self,
        ssa: &Ssa,
        function: &Function,
        brillig: &Brillig,
    ) -> Result<Option<GeneratedAcir>, RuntimeError> {
        match function.runtime() {
            RuntimeType::Acir(inline_type) => {
                match inline_type {
                    InlineType::Inline => {
                        if function.id() != ssa.main_id {
                            panic!("ACIR function should have been inlined earlier if not marked otherwise");
                        }
                    }
                    InlineType::NoPredicates => {
                        panic!("All ACIR functions marked with #[no_predicates] should be inlined before ACIR gen. This is an SSA exclusive codegen attribute");
                    }
                    InlineType::Fold => {}
                }
                // We only want to convert entry point functions. This being `main` and those marked with `InlineType::Fold`
                Ok(Some(self.convert_acir_main(function, ssa, brillig)?))
            }
            RuntimeType::Brillig => {
                if function.id() == ssa.main_id {
                    Ok(Some(self.convert_brillig_main(function, brillig)?))
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
        brillig: &Brillig,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];
        let input_witness = self.convert_ssa_block_params(entry_block.parameters(), dfg)?;

        self.data_bus = dfg.data_bus.to_owned();
        let mut warnings = Vec::new();
        for instruction_id in entry_block.instructions() {
            warnings.extend(self.convert_ssa_instruction(*instruction_id, dfg, ssa, brillig)?);
        }

        warnings.extend(self.convert_ssa_return(entry_block.unwrap_terminator(), dfg)?);
        Ok(self.acir_context.finish(input_witness, warnings))
    }

    fn convert_brillig_main(
        mut self,
        main_func: &Function,
        brillig: &Brillig,
    ) -> Result<GeneratedAcir, RuntimeError> {
        let dfg = &main_func.dfg;

        let inputs = try_vecmap(dfg[main_func.entry_block()].parameters(), |param_id| {
            let typ = dfg.type_of_value(*param_id);
            self.create_value_from_type(&typ, &mut |this, _| Ok(this.acir_context.add_variable()))
        })?;
        let arguments = self.gen_brillig_parameters(dfg[main_func.entry_block()].parameters(), dfg);

        let witness_inputs = self.acir_context.extract_witness(&inputs);

        let outputs: Vec<AcirType> =
            vecmap(main_func.returns(), |result_id| dfg.type_of_value(*result_id).into());

        let code = self.gen_brillig_for(main_func, arguments.clone(), brillig)?;

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
            0,
            None,
        )?;
        self.shared_context.insert_generated_brillig(main_func.id(), arguments, 0, code);

        let output_vars: Vec<_> = output_values
            .iter()
            .flat_map(|value| value.clone().flatten())
            .map(|value| value.0)
            .collect();

        for acir_var in output_vars {
            self.acir_context.return_var(acir_var)?;
        }

        let generated_acir = self.acir_context.finish(witness_inputs, Vec::new());

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
                        typ.flattened_size()
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

    /// Get the BlockId corresponding to the ValueId
    /// If there is no matching BlockId, we create a new one.
    fn block_id(&mut self, value: &ValueId) -> BlockId {
        if let Some(block_id) = self.memory_blocks.get(value) {
            return *block_id;
        }
        let block_id = BlockId(self.max_block_id);
        self.max_block_id += 1;
        self.memory_blocks.insert(*value, block_id);
        block_id
    }

    /// Get the next BlockId for internal memory
    /// used during ACIR generation.
    /// This is useful for referencing information that can
    /// only be computed dynamically, such as the type structure
    /// of non-homogenous arrays.
    fn internal_block_id(&mut self, value: &ValueId) -> BlockId {
        if let Some(block_id) = self.internal_memory_blocks.get(value) {
            return *block_id;
        }
        let block_id = BlockId(self.max_block_id);
        self.max_block_id += 1;
        self.internal_memory_blocks.insert(*value, block_id);
        block_id
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
        if matches!(numeric_type, NumericType::Signed { .. } | NumericType::Unsigned { .. }) {
            self.acir_context.range_constrain_var(acir_var, numeric_type, None)?;
        }
        Ok(acir_var)
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        brillig: &Brillig,
    ) -> Result<Vec<SsaReport>, RuntimeError> {
        let instruction = &dfg[instruction_id];
        self.acir_context.set_call_stack(dfg.get_call_stack(instruction_id));
        let mut warnings = Vec::new();
        match instruction {
            Instruction::Binary(binary) => {
                let result_acir_var = self.convert_ssa_binary(binary, dfg)?;
                self.define_result_var(dfg, instruction_id, result_acir_var);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                let lhs = self.convert_numeric_value(*lhs, dfg)?;
                let rhs = self.convert_numeric_value(*rhs, dfg)?;

                let assert_payload = if let Some(error) = assert_message {
                    match error {
                        ConstrainError::Intrinsic(string) => {
                            Some(AssertionPayload::StaticString(string.clone()))
                        }
                        ConstrainError::UserDefined(error_selector, values) => {
                            if let Some(constant_string) = try_to_extract_string_from_error_payload(
                                *error_selector,
                                values,
                                dfg,
                            ) {
                                Some(AssertionPayload::StaticString(constant_string))
                            } else {
                                let acir_vars: Vec<_> = values
                                    .iter()
                                    .map(|value| self.convert_value(*value, dfg))
                                    .collect();

                                let expressions_or_memory =
                                    self.acir_context.vars_to_expressions_or_memory(&acir_vars)?;

                                Some(AssertionPayload::Dynamic(
                                    error_selector.as_u64(),
                                    expressions_or_memory,
                                ))
                            }
                        }
                    }
                } else {
                    None
                };

                self.acir_context.assert_eq_var(lhs, rhs, assert_payload)?;
            }
            Instruction::Cast(value_id, _) => {
                let acir_var = self.convert_numeric_value(*value_id, dfg)?;
                self.define_result_var(dfg, instruction_id, acir_var);
            }
            Instruction::Call { .. } => {
                let result_ids = dfg.instruction_results(instruction_id);
                warnings.extend(self.convert_ssa_call(
                    instruction,
                    dfg,
                    ssa,
                    brillig,
                    result_ids,
                )?);
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
            Instruction::EnableSideEffects { condition } => {
                let acir_var = self.convert_numeric_value(*condition, dfg)?;
                self.current_side_effects_enabled_var = acir_var;
            }
            Instruction::ArrayGet { .. } | Instruction::ArraySet { .. } => {
                self.handle_array_operation(instruction_id, dfg)?;
            }
            Instruction::Allocate => {
                unreachable!("Expected all allocate instructions to be removed before acir_gen")
            }
            Instruction::Store { .. } => {
                unreachable!("Expected all store instructions to be removed before acir_gen")
            }
            Instruction::Load { .. } => {
                unreachable!("Expected all load instructions to be removed before acir_gen")
            }
            Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => {
                // Do nothing. Only Brillig needs to worry about reference counted arrays
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                let acir_var = self.convert_numeric_value(*value, dfg)?;
                self.acir_context.range_constrain_var(
                    acir_var,
                    &NumericType::Unsigned { bit_size: *max_bit_size },
                    assert_message.clone(),
                )?;
            }
            Instruction::IfElse { .. } => {
                unreachable!("IfElse instruction remaining in acir-gen")
            }
        }

        self.acir_context.set_call_stack(CallStack::new());
        Ok(warnings)
    }

    fn convert_ssa_call(
        &mut self,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        brillig: &Brillig,
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
                                assert!(!matches!(inline_type, InlineType::Inline), "ICE: Got an ACIR function named {} that should have already been inlined", func.name());

                                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
                                let output_count = result_ids
                                    .iter()
                                    .map(|result_id| dfg.type_of_value(*result_id).flattened_size())
                                    .sum();

                                let acir_function_id = ssa
                                    .entry_point_to_generated_index
                                    .get(id)
                                    .expect("ICE: should have an associated final index");
                                let output_vars = self.acir_context.call_acir_function(
                                    *acir_function_id,
                                    inputs,
                                    output_count,
                                    self.current_side_effects_enabled_var,
                                )?;

                                let output_values =
                                    self.convert_vars_to_values(output_vars, dfg, result_ids);

                                self.handle_ssa_call_outputs(result_ids, output_values, dfg)?;
                            }
                            RuntimeType::Brillig => {
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
                                        .generated_brillig(*generated_pointer as usize);
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
                                    let code =
                                        self.gen_brillig_for(func, arguments.clone(), brillig)?;
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
                                assert_eq!(result_ids.len(), output_values.len(), "ICE: The number of Brillig output values should match the result ids in SSA");

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
                        return Err(RuntimeError::UnconstrainedOracleReturnToConstrained {
                            call_stack: self.acir_context.get_call_stack(),
                        })
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
                let array_id = dfg.resolve(*result_id);
                let block_id = self.block_id(&array_id);
                let array_typ = dfg.type_of_value(array_id);
                let len = if matches!(array_typ, Type::Array(_, _)) {
                    array_typ.flattened_size()
                } else {
                    Self::flattened_value_size(&output)
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

    fn gen_brillig_for(
        &self,
        func: &Function,
        arguments: Vec<BrilligParameter>,
        brillig: &Brillig,
    ) -> Result<GeneratedBrillig, InternalError> {
        // Create the entry point artifact
        let mut entry_point = BrilligContext::new_entry_point_artifact(
            arguments,
            BrilligFunctionContext::return_values(func),
            BrilligFunctionContext::function_id_to_function_label(func.id()),
        );
        // Link the entry point with all dependencies
        while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
            let artifact = &brillig.find_by_function_label(unresolved_fn_label.clone());
            let artifact = match artifact {
                Some(artifact) => artifact,
                None => {
                    return Err(InternalError::General {
                        message: format!("Cannot find linked fn {unresolved_fn_label}"),
                        call_stack: CallStack::new(),
                    })
                }
            };
            entry_point.link_with(artifact);
        }
        // Generate the final bytecode
        Ok(entry_point.finish())
    }

    /// Handles an ArrayGet or ArraySet instruction.
    /// To set an index of the array (and create a new array in doing so), pass Some(value) for
    /// store_value. To just retrieve an index of the array, pass None for store_value.
    fn handle_array_operation(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        let mut mutable_array_set = false;

        // Pass the instruction between array methods rather than the internal fields themselves
        let (array, index, store_value) = match dfg[instruction] {
            Instruction::ArrayGet { array, index } => (array, index, None),
            Instruction::ArraySet { array, index, value, mutable } => {
                mutable_array_set = mutable;
                (array, index, Some(value))
            }
            _ => {
                return Err(InternalError::Unexpected {
                    expected: "Instruction should be an ArrayGet or ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into())
            }
        };

        if self.handle_constant_index(instruction, dfg, index, array, store_value)? {
            return Ok(());
        }

        // Get an offset such that the type of the array at the offset is the same as the type at the 'index'
        // If we find one, we will use it when computing the index under the enable_side_effect predicate
        // If not, array_get(..) will use a fallback costing one multiplication in the worst case.
        // cf. https://github.com/noir-lang/noir/pull/4971
        let array_id = dfg.resolve(array);
        let array_typ = dfg.type_of_value(array_id);
        // For simplicity we compute the offset only for simple arrays
        let is_simple_array = dfg.instruction_results(instruction).len() == 1
            && can_omit_element_sizes_array(&array_typ);
        let offset = if is_simple_array {
            let result_type = dfg.type_of_value(dfg.instruction_results(instruction)[0]);
            match array_typ {
                Type::Array(item_type, _) | Type::Slice(item_type) => item_type
                    .iter()
                    .enumerate()
                    .find_map(|(index, typ)| (result_type == *typ).then_some(index)),
                _ => None,
            }
        } else {
            None
        };
        let (new_index, new_value) = self.convert_array_operation_inputs(
            array,
            dfg,
            index,
            store_value,
            offset.unwrap_or_default(),
        )?;

        if let Some(new_value) = new_value {
            self.array_set(instruction, new_index, new_value, dfg, mutable_array_set)?;
        } else {
            self.array_get(instruction, array, new_index, dfg, offset.is_none())?;
        }

        Ok(())
    }

    /// Handle constant index: if there is no predicate and we have the array values,
    /// we can perform the operation directly on the array
    fn handle_constant_index(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        index: ValueId,
        array_id: ValueId,
        store_value: Option<ValueId>,
    ) -> Result<bool, RuntimeError> {
        let index_const = dfg.get_numeric_constant(index);
        let value_type = dfg.type_of_value(array_id);
        // Compiler sanity checks
        assert!(
            !value_type.is_nested_slice(),
            "ICE: Nested slice type has reached ACIR generation"
        );
        let (Type::Array(_, _) | Type::Slice(_)) = &value_type else {
            unreachable!("ICE: expected array or slice type");
        };

        match self.convert_value(array_id, dfg) {
            AcirValue::Var(acir_var, _) => {
                return Err(RuntimeError::InternalError(InternalError::Unexpected {
                    expected: "an array value".to_string(),
                    found: format!("{acir_var:?}"),
                    call_stack: self.acir_context.get_call_stack(),
                }))
            }
            AcirValue::Array(array) => {
                if let Some(index_const) = index_const {
                    let array_size = array.len();
                    let index = match index_const.try_to_u64() {
                        Some(index_const) => index_const as usize,
                        None => {
                            let call_stack = self.acir_context.get_call_stack();
                            return Err(RuntimeError::TypeConversion {
                                from: "array index".to_string(),
                                into: "u64".to_string(),
                                call_stack,
                            });
                        }
                    };

                    if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var) {
                        // Report the error if side effects are enabled.
                        if index >= array_size {
                            let call_stack = self.acir_context.get_call_stack();
                            return Err(RuntimeError::IndexOutOfBounds {
                                index,
                                array_size,
                                call_stack,
                            });
                        } else {
                            let value = match store_value {
                                Some(store_value) => {
                                    let store_value = self.convert_value(store_value, dfg);
                                    AcirValue::Array(array.update(index, store_value))
                                }
                                None => array[index].clone(),
                            };

                            self.define_result(dfg, instruction, value);
                            return Ok(true);
                        }
                    }
                    // If there is a predicate and the index is not out of range, we can directly perform the read
                    else if index < array_size && store_value.is_none() {
                        self.define_result(dfg, instruction, array[index].clone());
                        return Ok(true);
                    }
                }
            }
            AcirValue::DynamicArray(_) => (),
        };

        Ok(false)
    }

    /// We need to properly setup the inputs for array operations in ACIR.
    /// From the original SSA values we compute the following AcirVars:
    /// - new_index is the index of the array. ACIR memory operations work with a flat memory, so we fully flattened the specified index
    ///     in case we have a nested array. The index for SSA array operations only represents the flattened index of the current array.
    ///     Thus internal array element type sizes need to be computed to accurately transform the index.
    /// - predicate_index is offset, or the index if the predicate is true
    /// - new_value is the optional value when the operation is an array_set
    ///     When there is a predicate, it is predicate*value + (1-predicate)*dummy, where dummy is the value of the array at the requested index.
    ///     It is a dummy value because in the case of a false predicate, the value stored at the requested index will be itself.
    fn convert_array_operation_inputs(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
        index: ValueId,
        store_value: Option<ValueId>,
        offset: usize,
    ) -> Result<(AcirVar, Option<AcirValue>), RuntimeError> {
        let (array_id, array_typ, block_id) = self.check_array_is_initialized(array, dfg)?;

        let index_var = self.convert_numeric_value(index, dfg)?;
        let index_var = self.get_flattened_index(&array_typ, array_id, index_var, dfg)?;

        // predicate_index = index*predicate + (1-predicate)*offset
        let offset = self.acir_context.add_constant(offset);
        let sub = self.acir_context.sub_var(index_var, offset)?;
        let pred = self.acir_context.mul_var(sub, self.current_side_effects_enabled_var)?;
        let predicate_index = self.acir_context.add_var(pred, offset)?;

        let new_value = if let Some(store) = store_value {
            let store_value = self.convert_value(store, dfg);
            if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var) {
                Some(store_value)
            } else {
                let store_type = dfg.type_of_value(store);

                let mut dummy_predicate_index = predicate_index;
                // We must setup the dummy value to match the type of the value we wish to store
                let dummy =
                    self.array_get_value(&store_type, block_id, &mut dummy_predicate_index)?;

                Some(self.convert_array_set_store_value(&store_value, &dummy)?)
            }
        } else {
            None
        };

        let new_index = if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var)
        {
            index_var
        } else {
            predicate_index
        };

        Ok((new_index, new_value))
    }

    fn convert_array_set_store_value(
        &mut self,
        store_value: &AcirValue,
        dummy_value: &AcirValue,
    ) -> Result<AcirValue, RuntimeError> {
        match (store_value, dummy_value) {
            (AcirValue::Var(store_var, _), AcirValue::Var(dummy_var, _)) => {
                let true_pred =
                    self.acir_context.mul_var(*store_var, self.current_side_effects_enabled_var)?;
                let one = self.acir_context.add_constant(FieldElement::one());
                let not_pred =
                    self.acir_context.sub_var(one, self.current_side_effects_enabled_var)?;
                let false_pred = self.acir_context.mul_var(not_pred, *dummy_var)?;
                // predicate*value + (1-predicate)*dummy
                let new_value = self.acir_context.add_var(true_pred, false_pred)?;
                Ok(AcirValue::Var(new_value, AcirType::field()))
            }
            (AcirValue::Array(values), AcirValue::Array(dummy_values)) => {
                let mut elements = im::Vector::new();

                assert_eq!(
                    values.len(),
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );
                for (val, dummy_val) in values.iter().zip(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (
                AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }),
                AcirValue::Array(dummy_values),
            ) => {
                let dummy_values = dummy_values
                    .into_iter()
                    .flat_map(|val| val.clone().flatten())
                    .map(|(var, typ)| AcirValue::Var(var, typ))
                    .collect::<Vec<_>>();

                assert_eq!(
                    *len,
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );

                let values = try_vecmap(0..*len, |i| {
                    let index_var = self.acir_context.add_constant(i);

                    let read = self.acir_context.read_from_memory(*block_id, &index_var)?;
                    Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
                })?;

                let mut elements = im::Vector::new();
                for (val, dummy_val) in values.iter().zip(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, &dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (AcirValue::DynamicArray(_), AcirValue::DynamicArray(_)) => {
                unimplemented!("ICE: setting a dynamic array not supported");
            }
            _ => {
                unreachable!("ICE: The store value and dummy value must match");
            }
        }
    }

    /// Generates a read opcode for the array
    /// `index_side_effect == false` means that we ensured `var_index` will have a type matching the value in the array
    fn array_get(
        &mut self,
        instruction: InstructionId,
        array: ValueId,
        mut var_index: AcirVar,
        dfg: &DataFlowGraph,
        mut index_side_effect: bool,
    ) -> Result<AcirValue, RuntimeError> {
        let (array_id, _, block_id) = self.check_array_is_initialized(array, dfg)?;
        let results = dfg.instruction_results(instruction);
        let res_typ = dfg.type_of_value(results[0]);

        // Get operations to call-data parameters are replaced by a get to the call-data-bus array
        if let Some(call_data) = self.data_bus.call_data {
            if self.data_bus.call_data_map.contains_key(&array_id) {
                // TODO: the block_id of call-data must be notified to the backend
                // TODO: should we do the same for return-data?
                let type_size = res_typ.flattened_size();
                let type_size =
                    self.acir_context.add_constant(FieldElement::from(type_size as i128));
                let offset = self.acir_context.mul_var(var_index, type_size)?;
                let bus_index = self.acir_context.add_constant(FieldElement::from(
                    self.data_bus.call_data_map[&array_id] as i128,
                ));
                let new_index = self.acir_context.add_var(offset, bus_index)?;
                return self.array_get(instruction, call_data, new_index, dfg, index_side_effect);
            }
        }

        // Compiler sanity check
        assert!(
            !res_typ.contains_slice_element(),
            "ICE: Nested slice result found during ACIR generation"
        );
        let mut value = self.array_get_value(&res_typ, block_id, &mut var_index)?;

        if let AcirValue::Var(value_var, typ) = &value {
            let array_id = dfg.resolve(array_id);
            let array_typ = dfg.type_of_value(array_id);
            if let (Type::Numeric(numeric_type), AcirType::NumericType(num)) =
                (array_typ.first(), typ)
            {
                if numeric_type.bit_size() <= num.bit_size() {
                    // first element is compatible
                    index_side_effect = false;
                }
            }
            // Fallback to multiplication if the index side_effects have not already been handled
            if index_side_effect {
                // Set the value to 0 if current_side_effects is 0, to ensure it fits in any value type
                value = AcirValue::Var(
                    self.acir_context.mul_var(*value_var, self.current_side_effects_enabled_var)?,
                    typ.clone(),
                );
            }
        }

        self.define_result(dfg, instruction, value.clone());

        Ok(value)
    }

    fn array_get_value(
        &mut self,
        ssa_type: &Type,
        block_id: BlockId,
        var_index: &mut AcirVar,
    ) -> Result<AcirValue, RuntimeError> {
        let one = self.acir_context.add_constant(FieldElement::one());
        match ssa_type.clone() {
            Type::Numeric(numeric_type) => {
                // Read the value from the array at the specified index
                let read = self.acir_context.read_from_memory(block_id, var_index)?;

                // Increment the var_index in case of a nested array
                *var_index = self.acir_context.add_var(*var_index, one)?;

                let typ = AcirType::NumericType(numeric_type);
                Ok(AcirValue::Var(read, typ))
            }
            Type::Array(element_types, len) => {
                let mut values = Vector::new();
                for _ in 0..len {
                    for typ in element_types.as_ref() {
                        values.push_back(self.array_get_value(typ, block_id, var_index)?);
                    }
                }
                Ok(AcirValue::Array(values))
            }
            _ => unreachable!("ICE: Expected an array or numeric but got {ssa_type:?}"),
        }
    }

    /// If `mutate_array` is:
    /// - true: Mutate the array directly
    /// - false: Copy the array and generates a write opcode on the new array. This is
    ///          generally very inefficient and should be avoided if possible. Currently
    ///          this is controlled by SSA's array set optimization pass.
    fn array_set(
        &mut self,
        instruction: InstructionId,
        mut var_index: AcirVar,
        store_value: AcirValue,
        dfg: &DataFlowGraph,
        mutate_array: bool,
    ) -> Result<(), RuntimeError> {
        // Pass the instruction between array methods rather than the internal fields themselves
        let array = match dfg[instruction] {
            Instruction::ArraySet { array, .. } => array,
            _ => {
                return Err(InternalError::Unexpected {
                    expected: "Instruction should be an ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into())
            }
        };

        let (array_id, array_typ, block_id) = self.check_array_is_initialized(array, dfg)?;

        // Every array has a length in its type, so we fetch that from
        // the SSA IR.
        //
        // A slice's size must be fetched from the SSA value that represents the slice.
        // However, this size is simply the capacity of a slice. The capacity is dependent upon the witness
        // and may contain data for which we want to restrict access. The true slice length is tracked in a
        // a separate SSA value and restrictions on slice indices should be generated elsewhere in the SSA.
        let array_len = if !array_typ.contains_slice_element() {
            array_typ.flattened_size()
        } else {
            self.flattened_slice_size(array_id, dfg)
        };

        // Since array_set creates a new array, we create a new block ID for this
        // array, unless map_array is true. In that case, we operate directly on block_id
        // and we do not create a new block ID.
        let result_id = dfg
            .instruction_results(instruction)
            .first()
            .expect("Array set does not have one result");
        let result_block_id;
        if mutate_array {
            self.memory_blocks.insert(*result_id, block_id);
            result_block_id = block_id;
        } else {
            // Initialize the new array with the values from the old array
            result_block_id = self.block_id(result_id);
            self.copy_dynamic_array(block_id, result_block_id, array_len)?;
        }

        self.array_set_value(&store_value, result_block_id, &mut var_index)?;

        let element_type_sizes = if !can_omit_element_sizes_array(&array_typ) {
            let acir_value = self.convert_value(array_id, dfg);
            Some(self.init_element_type_sizes_array(
                &array_typ,
                array_id,
                Some(&acir_value),
                dfg,
            )?)
        } else {
            None
        };

        let value_types = self.convert_value(array_id, dfg).flat_numeric_types();
        // Compiler sanity check
        assert_eq!(value_types.len(), array_len, "ICE: The length of the flattened type array should match the length of the dynamic array");

        let result_value = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: array_len,
            value_types,
            element_type_sizes,
        });
        self.define_result(dfg, instruction, result_value);
        Ok(())
    }

    fn array_set_value(
        &mut self,
        value: &AcirValue,
        block_id: BlockId,
        var_index: &mut AcirVar,
    ) -> Result<(), RuntimeError> {
        let one = self.acir_context.add_constant(FieldElement::one());
        match value {
            AcirValue::Var(store_var, _) => {
                // Write the new value into the new array at the specified index
                self.acir_context.write_to_memory(block_id, var_index, store_var)?;
                // Increment the var_index in case of a nested array
                *var_index = self.acir_context.add_var(*var_index, one)?;
            }
            AcirValue::Array(values) => {
                for value in values {
                    self.array_set_value(value, block_id, var_index)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id: inner_block_id, len, .. }) => {
                let values = try_vecmap(0..*len, |i| {
                    let index_var = self.acir_context.add_constant(i);

                    let read = self.acir_context.read_from_memory(*inner_block_id, &index_var)?;
                    Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
                })?;
                self.array_set_value(&AcirValue::Array(values.into()), block_id, var_index)?;
            }
        }
        Ok(())
    }

    fn check_array_is_initialized(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<(ValueId, Type, BlockId), RuntimeError> {
        // Fetch the internal SSA ID for the array
        let array_id = dfg.resolve(array);

        let array_typ = dfg.type_of_value(array_id);

        // Use the SSA ID to get or create its block ID
        let block_id = self.block_id(&array_id);

        // Check if the array has already been initialized in ACIR gen
        // if not, we initialize it using the values from SSA
        let already_initialized = self.initialized_arrays.contains(&block_id);
        if !already_initialized {
            let value = &dfg[array_id];
            match value {
                Value::Array { .. } | Value::Instruction { .. } => {
                    let value = self.convert_value(array_id, dfg);
                    let len = if !array_typ.contains_slice_element() {
                        array_typ.flattened_size()
                    } else {
                        self.flattened_slice_size(array_id, dfg)
                    };
                    self.initialize_array(block_id, len, Some(value))?;
                }
                _ => {
                    return Err(InternalError::General {
                        message: format!("Array {array_id} should be initialized"),
                        call_stack: self.acir_context.get_call_stack(),
                    }
                    .into());
                }
            }
        }

        Ok((array_id, array_typ, block_id))
    }

    fn init_element_type_sizes_array(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        supplied_acir_value: Option<&AcirValue>,
        dfg: &DataFlowGraph,
    ) -> Result<BlockId, RuntimeError> {
        let element_type_sizes = self.internal_block_id(&array_id);
        // Check whether an internal type sizes array has already been initialized
        // Need to look into how to optimize for slices as this could lead to different element type sizes
        // for different slices that do not have consistent sizes
        if self.initialized_arrays.contains(&element_type_sizes) {
            return Ok(element_type_sizes);
        }

        let mut flat_elem_type_sizes = Vec::new();
        flat_elem_type_sizes.push(0);
        match array_typ {
            Type::Array(_, _) | Type::Slice(_) => {
                match &dfg[array_id] {
                    Value::Array { array, .. } => {
                        for (i, value) in array.iter().enumerate() {
                            flat_elem_type_sizes.push(
                                self.flattened_slice_size(*value, dfg) + flat_elem_type_sizes[i],
                            );
                        }
                    }
                    Value::Instruction { .. } | Value::Param { .. } => {
                        // An instruction representing the slice means it has been processed previously during ACIR gen.
                        // Use the previously defined result of an array operation to fetch the internal type information.
                        let array_acir_value = &self.convert_value(array_id, dfg);
                        let array_acir_value = supplied_acir_value.unwrap_or(array_acir_value);
                        match array_acir_value {
                            AcirValue::DynamicArray(AcirDynamicArray {
                                element_type_sizes: inner_elem_type_sizes,
                                ..
                            }) => {
                                if let Some(inner_elem_type_sizes) = inner_elem_type_sizes {
                                    if self.initialized_arrays.contains(inner_elem_type_sizes) {
                                        let type_sizes_array_len = *self.internal_mem_block_lengths.get(inner_elem_type_sizes).ok_or_else(||
                                            InternalError::General {
                                                message: format!("Array {array_id}'s inner element type sizes array does not have a tracked length"),
                                                call_stack: self.acir_context.get_call_stack(),
                                            }
                                        )?;
                                        self.copy_dynamic_array(
                                            *inner_elem_type_sizes,
                                            element_type_sizes,
                                            type_sizes_array_len,
                                        )?;
                                        self.internal_mem_block_lengths
                                            .insert(element_type_sizes, type_sizes_array_len);
                                        return Ok(element_type_sizes);
                                    } else {
                                        return Err(InternalError::General {
                                            message: format!("Array {array_id}'s inner element type sizes array should be initialized"),
                                            call_stack: self.acir_context.get_call_stack(),
                                        }
                                        .into());
                                    }
                                }
                            }
                            AcirValue::Array(values) => {
                                for (i, value) in values.iter().enumerate() {
                                    flat_elem_type_sizes.push(
                                        Self::flattened_value_size(value) + flat_elem_type_sizes[i],
                                    );
                                }
                            }
                            _ => {
                                return Err(InternalError::Unexpected {
                                    expected: "AcirValue::DynamicArray or AcirValue::Array"
                                        .to_owned(),
                                    found: format!("{:?}", array_acir_value),
                                    call_stack: self.acir_context.get_call_stack(),
                                }
                                .into())
                            }
                        }
                    }
                    _ => {
                        return Err(InternalError::Unexpected {
                            expected: "array or instruction".to_owned(),
                            found: format!("{:?}", &dfg[array_id]),
                            call_stack: self.acir_context.get_call_stack(),
                        }
                        .into())
                    }
                };
            }
            _ => {
                return Err(InternalError::Unexpected {
                    expected: "array or slice".to_owned(),
                    found: array_typ.to_string(),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into());
            }
        }

        // The final array should will the flattened index at each outer array index
        let init_values = vecmap(flat_elem_type_sizes, |type_size| {
            let var = self.acir_context.add_constant(type_size);
            AcirValue::Var(var, AcirType::field())
        });
        let element_type_sizes_len = init_values.len();
        self.initialize_array(
            element_type_sizes,
            element_type_sizes_len,
            Some(AcirValue::Array(init_values.into())),
        )?;

        self.internal_mem_block_lengths.insert(element_type_sizes, element_type_sizes_len);

        Ok(element_type_sizes)
    }

    fn copy_dynamic_array(
        &mut self,
        source: BlockId,
        destination: BlockId,
        array_len: usize,
    ) -> Result<(), RuntimeError> {
        let init_values = try_vecmap(0..array_len, |i| {
            let index_var = self.acir_context.add_constant(i);

            let read = self.acir_context.read_from_memory(source, &index_var)?;
            Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
        })?;
        let array: im::Vector<AcirValue> = init_values.into();
        self.initialize_array(destination, array_len, Some(AcirValue::Array(array)))?;
        Ok(())
    }

    fn get_flattened_index(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        var_index: AcirVar,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        if !can_omit_element_sizes_array(array_typ) {
            let element_type_sizes =
                self.init_element_type_sizes_array(array_typ, array_id, None, dfg)?;

            let predicate_index =
                self.acir_context.mul_var(var_index, self.current_side_effects_enabled_var)?;

            self.acir_context
                .read_from_memory(element_type_sizes, &predicate_index)
                .map_err(RuntimeError::from)
        } else {
            Ok(var_index)
        }
    }

    fn flattened_slice_size(&mut self, array_id: ValueId, dfg: &DataFlowGraph) -> usize {
        let mut size = 0;
        match &dfg[array_id] {
            Value::Array { array, .. } => {
                // The array is going to be the flattened outer array
                // Flattened slice size from SSA value does not need to be multiplied by the len
                for value in array {
                    size += self.flattened_slice_size(*value, dfg);
                }
            }
            Value::NumericConstant { .. } => {
                size += 1;
            }
            Value::Instruction { .. } => {
                let array_acir_value = self.convert_value(array_id, dfg);
                size += Self::flattened_value_size(&array_acir_value);
            }
            Value::Param { .. } => {
                let array_acir_value = self.convert_value(array_id, dfg);
                size += Self::flattened_value_size(&array_acir_value);
            }
            _ => {
                unreachable!("ICE: Unexpected SSA value when computing the slice size");
            }
        }
        size
    }

    fn flattened_value_size(value: &AcirValue) -> usize {
        let mut size = 0;
        match value {
            AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => {
                size += len;
            }
            AcirValue::Var(_, _) => {
                size += 1;
            }
            AcirValue::Array(values) => {
                for value in values {
                    size += Self::flattened_value_size(value);
                }
            }
        }
        size
    }

    /// Initializes an array with the given values and caches the fact that we
    /// have initialized this array.
    fn initialize_array(
        &mut self,
        array: BlockId,
        len: usize,
        value: Option<AcirValue>,
    ) -> Result<(), InternalError> {
        let databus = if self.data_bus.call_data.is_some()
            && self.block_id(&self.data_bus.call_data.unwrap()) == array
        {
            BlockType::CallData
        } else if self.data_bus.return_data.is_some()
            && self.block_id(&self.data_bus.return_data.unwrap()) == array
        {
            BlockType::ReturnData
        } else {
            BlockType::Memory
        };
        self.acir_context.initialize_array(array, len, value, databus)?;
        self.initialized_arrays.insert(array);
        Ok(())
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
    fn convert_ssa_return(
        &mut self,
        terminator: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) -> Result<Vec<SsaReport>, InternalError> {
        let (return_values, call_stack) = match terminator {
            TerminatorInstruction::Return { return_values, call_stack } => {
                (return_values, call_stack)
            }
            // TODO(https://github.com/noir-lang/noir/issues/4616): Enable recursion on foldable/non-inlined ACIR functions
            _ => unreachable!("ICE: Program must have a singular return"),
        };

        // The return value may or may not be an array reference. Calling `flatten_value_list`
        // will expand the array if there is one.
        let return_acir_vars = self.flatten_value_list(return_values, dfg)?;
        let mut warnings = Vec::new();
        for (acir_var, is_databus) in return_acir_vars {
            if self.acir_context.is_constant(&acir_var) {
                warnings.push(SsaReport::Warning(InternalWarning::ReturnConstant {
                    call_stack: call_stack.clone(),
                }));
            }
            if !is_databus {
                // We do not return value for the data bus.
                self.acir_context.return_var(acir_var)?;
            }
        }
        Ok(warnings)
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
        let value_id = dfg.resolve(value_id);
        let value = &dfg[value_id];
        if let Some(acir_value) = self.ssa_values.get(&value_id) {
            return acir_value.clone();
        }

        let acir_value = match value {
            Value::NumericConstant { constant, typ } => {
                AcirValue::Var(self.acir_context.add_constant(*constant), typ.into())
            }
            Value::Array { array, .. } => {
                let elements = array.iter().map(|element| self.convert_value(*element, dfg));
                AcirValue::Array(elements.collect())
            }
            Value::Intrinsic(..) => todo!(),
            Value::Function(function_id) => {
                // This conversion is for debugging support only, to allow the
                // debugging instrumentation code to work. Taking the reference
                // of a function in ACIR is useless.
                let id = self.acir_context.add_constant(function_id.to_usize());
                AcirValue::Var(id, AcirType::field())
            }
            Value::ForeignFunction(_) => unimplemented!(
                "Oracle calls directly in constrained functions are not yet available."
            ),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value_id} {value:?}")
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
    ) -> Result<AcirVar, RuntimeError> {
        let lhs = self.convert_numeric_value(binary.lhs, dfg)?;
        let rhs = self.convert_numeric_value(binary.rhs, dfg)?;

        let binary_type = self.type_of_binary_operation(binary, dfg);
        match &binary_type {
            Type::Numeric(NumericType::Unsigned { bit_size })
            | Type::Numeric(NumericType::Signed { bit_size }) => {
                // Conservative max bit size that is small enough such that two operands can be
                // multiplied and still fit within the field modulus. This is necessary for the
                // truncation technique: result % 2^bit_size to be valid.
                let max_integer_bit_size = FieldElement::max_num_bits() / 2;
                if *bit_size > max_integer_bit_size {
                    return Err(RuntimeError::UnsupportedIntegerSize {
                        num_bits: *bit_size,
                        max_num_bits: max_integer_bit_size,
                        call_stack: self.acir_context.get_call_stack(),
                    });
                }
            }
            _ => {}
        }

        let binary_type = AcirType::from(binary_type);
        let bit_count = binary_type.bit_size();
        let num_type = binary_type.to_numeric_type();
        let result = match binary.operator {
            BinaryOp::Add => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub => self.acir_context.sub_var(lhs, rhs),
            BinaryOp::Mul => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(
                lhs,
                rhs,
                binary_type.clone(),
                self.current_side_effects_enabled_var,
            ),
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
            BinaryOp::Mod => self.acir_context.modulo_var(
                lhs,
                rhs,
                bit_count,
                self.current_side_effects_enabled_var,
            ),
            BinaryOp::Shl | BinaryOp::Shr => unreachable!(
                "ICE - bit shift operators do not exist in ACIR and should have been replaced"
            ),
        }?;

        if let NumericType::Unsigned { bit_size } = &num_type {
            // Check for integer overflow
            self.check_unsigned_overflow(
                result,
                *bit_size,
                binary.lhs,
                binary.rhs,
                dfg,
                binary.operator,
            )?;
        }

        Ok(result)
    }

    /// Adds a range check against the bit size of the result of addition, subtraction or multiplication
    fn check_unsigned_overflow(
        &mut self,
        result: AcirVar,
        bit_size: u32,
        lhs: ValueId,
        rhs: ValueId,
        dfg: &DataFlowGraph,
        op: BinaryOp,
    ) -> Result<(), RuntimeError> {
        // We try to optimize away operations that are guaranteed not to overflow
        let max_lhs_bits = dfg.get_value_max_num_bits(lhs);
        let max_rhs_bits = dfg.get_value_max_num_bits(rhs);

        let msg = match op {
            BinaryOp::Add => {
                if std::cmp::max(max_lhs_bits, max_rhs_bits) < bit_size {
                    // `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    return Ok(());
                }
                "attempt to add with overflow".to_string()
            }
            BinaryOp::Sub => {
                if dfg.is_constant(lhs) && max_lhs_bits > max_rhs_bits {
                    // `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs > 0`
                    // Note strict inequality as `rhs > lhs` while `max_lhs_bits == max_rhs_bits` is possible.
                    return Ok(());
                }
                "attempt to subtract with overflow".to_string()
            }
            BinaryOp::Mul => {
                if bit_size == 1 || max_lhs_bits + max_rhs_bits <= bit_size {
                    // Either performing boolean multiplication (which cannot overflow),
                    // or `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                    return Ok(());
                }
                "attempt to multiply with overflow".to_string()
            }
            _ => return Ok(()),
        };

        let with_pred = self.acir_context.mul_var(result, self.current_side_effects_enabled_var)?;
        self.acir_context.range_constrain_var(
            with_pred,
            &NumericType::Unsigned { bit_size },
            Some(msg),
        )?;
        Ok(())
    }

    /// Operands in a binary operation are checked to have the same type.
    ///
    /// In Noir, binary operands should have the same type due to the language
    /// semantics.
    ///
    /// There are some edge cases to consider:
    /// - Constants are not explicitly type casted, so we need to check for this and
    /// return the type of the other operand, if we have a constant.
    /// - 0 is not seen as `Field 0` but instead as `Unit 0`
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
        max_bit_size: u32,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        let mut var = self.convert_numeric_value(value_id, dfg)?;
        match &dfg[value_id] {
            Value::Instruction { instruction, .. } => {
                if matches!(
                    &dfg[*instruction],
                    Instruction::Binary(Binary { operator: BinaryOp::Sub, .. })
                ) {
                    // Subtractions must first have the integer modulus added before truncation can be
                    // applied. This is done in order to prevent underflow.
                    let integer_modulus = self.acir_context.add_constant(2_u128.pow(bit_size));
                    var = self.acir_context.add_var(var, integer_modulus)?;
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
                    sum + dfg.try_get_array_length(*result_id).unwrap_or(1)
                });

                let vars = self.acir_context.black_box_function(black_box, inputs, output_count)?;

                Ok(self.convert_vars_to_values(vars, dfg, result_ids))
            }
            Intrinsic::ApplyRangeConstraint => {
                unreachable!("ICE: `Intrinsic::ApplyRangeConstraint` calls should be transformed into an `Instruction::RangeCheck`");
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let radix = self.convert_value(arguments[1], dfg).into_var()?;
                let limb_size = self.convert_value(arguments[2], dfg).into_var()?;

                let result_type = Self::array_element_type(dfg, result_ids[1]);

                self.acir_context.radix_decompose(endian, field, radix, limb_size, result_type)
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let bit_size = self.convert_value(arguments[1], dfg).into_var()?;

                let result_type = Self::array_element_type(dfg, result_ids[1]);

                self.acir_context.bit_decompose(endian, field, bit_size, result_type)
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
                let (slice_contents, slice_typ, block_id) =
                    self.check_array_is_initialized(arguments[0], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let result_block_id = self.block_id(&result_ids[1]);
                let acir_value = self.convert_value(slice_contents, dfg);

                let array_len = if !slice_typ.contains_slice_element() {
                    slice_typ.flattened_size()
                } else {
                    self.flattened_slice_size(slice_contents, dfg)
                };
                let slice_length = self.acir_context.add_constant(array_len);
                self.copy_dynamic_array(block_id, result_block_id, array_len)?;

                let element_type_sizes = if !can_omit_element_sizes_array(&slice_typ) {
                    Some(self.init_element_type_sizes_array(
                        &slice_typ,
                        slice_contents,
                        Some(&acir_value),
                        dfg,
                    )?)
                } else {
                    None
                };

                let value_types = self.convert_value(slice_contents, dfg).flat_numeric_types();
                assert!(
                    array_len == value_types.len(),
                    "AsSlice: unexpected length difference: {:?} != {:?}",
                    array_len,
                    value_types.len()
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: value_types.len(),
                    value_types,
                    element_type_sizes,
                });
                Ok(vec![AcirValue::Var(slice_length, AcirType::field()), result])
            }
            Intrinsic::SlicePushBack => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let (slice_contents, slice_typ, _) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let mut new_elem_size = Self::flattened_value_size(&slice);

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;

                let elements_to_push = &arguments[2..];
                // We must directly push back elements for non-nested slices
                for elem in elements_to_push {
                    let element = self.convert_value(*elem, dfg);

                    new_elem_size += Self::flattened_value_size(&element);
                    new_slice.push_back(element);
                }

                // Increase the slice length by one to enable accessing more elements in the slice.
                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let new_slice_val = AcirValue::Array(new_slice);
                let result_block_id = self.block_id(&result_ids[1]);
                self.initialize_array(result_block_id, new_elem_size, Some(new_slice_val.clone()))?;
                // The previous slice length represents the index we want to write into.
                let mut var_index = slice_length;
                // Write the elements we wish to push back directly.
                // The slice's underlying array value should already be filled with dummy data
                // to enable this write to be within bounds.
                // The dummy data is either attached during SSA gen or in this match case for non-nested slices.
                // These values can then be accessed due to the increased dynamic slice length.
                for elem in elements_to_push {
                    let element = self.convert_value(*elem, dfg);
                    self.array_set_value(&element, result_block_id, &mut var_index)?;
                }

                let element_type_sizes = if !can_omit_element_sizes_array(&slice_typ) {
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
                    new_elem_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: new_elem_size,
                    value_types,
                    element_type_sizes,
                });
                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), result])
            }
            Intrinsic::SlicePushFront => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;

                let (slice_contents, slice_typ, _) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice: AcirValue = self.convert_value(slice_contents, dfg);
                let mut new_slice_size = Self::flattened_value_size(&slice);

                // Increase the slice length by one to enable accessing more elements in the slice.
                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;

                let elements_to_push = &arguments[2..];
                let mut elem_size = 0;
                // We must directly push front elements for non-nested slices
                for elem in elements_to_push.iter().rev() {
                    let element = self.convert_value(*elem, dfg);

                    elem_size += Self::flattened_value_size(&element);
                    new_slice.push_front(element);
                }
                new_slice_size += elem_size;

                let new_slice_val = AcirValue::Array(new_slice.clone());

                let result_block_id = self.block_id(&result_ids[1]);
                self.initialize_array(
                    result_block_id,
                    new_slice_size,
                    Some(new_slice_val.clone()),
                )?;

                let element_type_sizes = if !can_omit_element_sizes_array(&slice_typ) {
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
                    new_slice_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: new_slice_size,
                    value_types,
                    element_type_sizes,
                });

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), result])
            }
            Intrinsic::SlicePopBack => {
                // arguments = [slice_length, slice_contents]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;
                // For a pop back operation we want to fetch from the `length - 1` as this is the
                // last valid index that can be accessed in a slice. After the pop back operation
                // the elements stored at that index will no longer be able to be accessed.
                let mut var_index = new_slice_length;

                let (slice_contents, slice_typ, block_id) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let mut popped_elements = Vec::new();
                for res in &result_ids[2..] {
                    let elem =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
                    popped_elements.push(elem);
                }

                let slice = self.convert_value(slice_contents, dfg);
                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;

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

                let (slice_contents, slice_typ, block_id) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let slice = self.convert_value(slice_contents, dfg);

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;

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
                    let elem_size = Self::flattened_value_size(&element);
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

                let (slice_contents, slice_typ, block_id) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let insert_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let slice_size = Self::flattened_value_size(&slice);

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
                    let elem_size = Self::flattened_value_size(&element);
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

                let element_type_sizes = if !can_omit_element_sizes_array(&slice_typ) {
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

                let (slice_contents, slice_typ, block_id) =
                    self.check_array_is_initialized(arguments[1], dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let remove_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let slice_size = Self::flattened_value_size(&slice);

                let mut new_slice = Vector::new();
                self.slice_intrinsic_input(&mut new_slice, slice)?;

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
                    let elem_size = Self::flattened_value_size(&element);
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
                let element_type_sizes = if !can_omit_element_sizes_array(&slice_typ) {
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
            _ => todo!("expected a black box function"),
        }
    }

    fn slice_intrinsic_input(
        &mut self,
        old_slice: &mut Vector<AcirValue>,
        input: AcirValue,
    ) -> Result<(), RuntimeError> {
        match input {
            AcirValue::Var(_, _) => {
                old_slice.push_back(input);
            }
            AcirValue::Array(vars) => {
                for var in vars {
                    self.slice_intrinsic_input(old_slice, var)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }) => {
                for i in 0..len {
                    // We generate witnesses corresponding to the array values
                    let index_var = self.acir_context.add_constant(i);

                    let value_read_var =
                        self.acir_context.read_from_memory(block_id, &index_var)?;
                    let value_read = AcirValue::Var(value_read_var, AcirType::field());

                    old_slice.push_back(value_read);
                }
            }
        }
        Ok(())
    }

    /// Given an array value, return the numerical type of its element.
    /// Panics if the given value is not an array or has a non-numeric element type.
    fn array_element_type(dfg: &DataFlowGraph, value: ValueId) -> AcirType {
        match dfg.type_of_value(value) {
            Type::Array(elements, _) => {
                assert_eq!(elements.len(), 1);
                (&elements[0]).into()
            }
            Type::Slice(elements) => {
                assert_eq!(elements.len(), 1);
                (&elements[0]).into()
            }
            _ => unreachable!("Expected array type"),
        }
    }

    /// Maps an ssa value list, for which some values may be references to arrays, by inlining
    /// the `AcirVar`s corresponding to the contents of each array into the list of `AcirVar`s
    /// that correspond to other values.
    fn flatten_value_list(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<(AcirVar, bool)>, InternalError> {
        let mut acir_vars = Vec::with_capacity(arguments.len());
        for value_id in arguments {
            let is_databus = if let Some(return_databus) = self.data_bus.return_data {
                dfg[*value_id] == dfg[return_databus]
            } else {
                false
            };
            let value = self.convert_value(*value_id, dfg);
            acir_vars.append(
                &mut self
                    .acir_context
                    .flatten(value)?
                    .iter()
                    .map(|(var, _)| (*var, is_databus))
                    .collect(),
            );
        }
        Ok(acir_vars)
    }

    /// Convert a Vec<AcirVar> into a Vec<AcirValue> using the given result ids.
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

// We can omit the element size array for arrays which don't contain arrays or slices.
fn can_omit_element_sizes_array(array_typ: &Type) -> bool {
    let types = match array_typ {
        Type::Array(types, _) | Type::Slice(types) => types,
        _ => panic!("ICE: expected array or slice type"),
    };

    !types.iter().any(|typ| typ.contains_an_array())
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{
        acir::{
            circuit::{Opcode, OpcodeLocation},
            native_types::Witness,
        },
        FieldElement,
    };
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::{
        brillig::Brillig,
        ssa::{
            acir_gen::acir_ir::generated_acir::BrilligStdlibFunc,
            function_builder::FunctionBuilder,
            ir::{function::FunctionId, instruction::BinaryOp, map::Id, types::Type},
        },
    };

    fn build_basic_foo_with_return(
        builder: &mut FunctionBuilder,
        foo_id: FunctionId,
        // `InlineType` can only exist on ACIR functions, so if the option is `None` we should generate a Brillig function
        inline_type: Option<InlineType>,
    ) {
        // fn foo f1 {
        // b0(v0: Field, v1: Field):
        //     v2 = eq v0, v1
        //     constrain v2 == u1 0
        //     return v0
        // }
        if let Some(inline_type) = inline_type {
            builder.new_function("foo".into(), foo_id, inline_type);
        } else {
            builder.new_brillig_function("foo".into(), foo_id);
        }
        let foo_v0 = builder.add_parameter(Type::field());
        let foo_v1 = builder.add_parameter(Type::field());

        let foo_equality_check = builder.insert_binary(foo_v0, BinaryOp::Eq, foo_v1);
        let zero = builder.numeric_constant(0u128, Type::unsigned(1));
        builder.insert_constrain(foo_equality_check, zero, None);
        builder.terminate_with_return(vec![foo_v0]);
    }

    /// Check that each `InlineType` which prevents inlining functions generates code in the same manner
    #[test]
    fn basic_calls_fold() {
        basic_call_with_outputs_assert(InlineType::Fold);
        call_output_as_next_call_input(InlineType::Fold);
        basic_nested_call(InlineType::Fold);
    }

    #[test]
    #[should_panic]
    fn basic_calls_no_predicates() {
        basic_call_with_outputs_assert(InlineType::NoPredicates);
        call_output_as_next_call_input(InlineType::NoPredicates);
        basic_nested_call(InlineType::NoPredicates);
    }

    #[test]
    #[should_panic]
    fn call_without_inline_attribute() {
        basic_call_with_outputs_assert(InlineType::Inline);
    }

    fn basic_call_with_outputs_assert(inline_type: InlineType) {
        // acir(inline) fn main f0 {
        //     b0(v0: Field, v1: Field):
        //       v2 = call f1(v0, v1)
        //       v3 = call f1(v0, v1)
        //       constrain v2 == v3
        //       return
        //     }
        // acir(fold) fn foo f1 {
        //     b0(v0: Field, v1: Field):
        //       v2 = eq v0, v1
        //       constrain v2 == u1 0
        //       return v0
        //     }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::field());
        let main_v1 = builder.add_parameter(Type::field());

        let foo_id = Id::test_new(1);
        let foo = builder.import_function(foo_id);
        let main_call1_results =
            builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        let main_call2_results =
            builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
        builder.terminate_with_return(vec![]);

        build_basic_foo_with_return(&mut builder, foo_id, Some(inline_type));

        let ssa = builder.finish();

        let (acir_functions, _, _) = ssa
            .into_acir(&Brillig::default())
            .expect("Should compile manually written SSA into ACIR");
        // Expected result:
        // main f0
        // GeneratedAcir {
        //     ...
        //     opcodes: [
        //         CALL func 1: inputs: [Witness(0), Witness(1)], outputs: [Witness(2)],
        //         CALL func 1: inputs: [Witness(0), Witness(1)], outputs: [Witness(3)],
        //         EXPR [ (1, _2) (-1, _3) 0 ],
        //     ],
        //     return_witnesses: [],
        //     input_witnesses: [
        //         Witness(
        //             0,
        //         ),
        //         Witness(
        //             1,
        //         ),
        //     ],
        //     ...
        // }
        // foo f1
        // GeneratedAcir {
        //     ...
        //     opcodes: [
        //         Same as opcodes as the expected result of `basic_call_codegen`
        //     ],
        //     return_witnesses: [
        //         Witness(
        //             0,
        //         ),
        //     ],
        //     input_witnesses: [
        //         Witness(
        //             0,
        //         ),
        //         Witness(
        //             1,
        //         ),
        //     ],
        //     ...
        // },

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo`");

        check_call_opcode(&main_opcodes[0], 1, vec![Witness(0), Witness(1)], vec![Witness(2)]);
        check_call_opcode(&main_opcodes[1], 1, vec![Witness(0), Witness(1)], vec![Witness(3)]);

        if let Opcode::AssertZero(expr) = &main_opcodes[2] {
            assert_eq!(expr.linear_combinations[0].0, FieldElement::from(1u128));
            assert_eq!(expr.linear_combinations[0].1, Witness(2));

            assert_eq!(expr.linear_combinations[1].0, FieldElement::from(-1i128));
            assert_eq!(expr.linear_combinations[1].1, Witness(3));
            assert_eq!(expr.q_c, FieldElement::from(0u128));
        }
    }

    fn call_output_as_next_call_input(inline_type: InlineType) {
        // acir(inline) fn main f0 {
        //     b0(v0: Field, v1: Field):
        //       v3 = call f1(v0, v1)
        //       v4 = call f1(v3, v1)
        //       constrain v3 == v4
        //       return
        //     }
        // acir(fold) fn foo f1 {
        //     b0(v0: Field, v1: Field):
        //       v2 = eq v0, v1
        //       constrain v2 == u1 0
        //       return v0
        //     }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::field());
        let main_v1 = builder.add_parameter(Type::field());

        let foo_id = Id::test_new(1);
        let foo = builder.import_function(foo_id);
        let main_call1_results =
            builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        let main_call2_results = builder
            .insert_call(foo, vec![main_call1_results[0], main_v1], vec![Type::field()])
            .to_vec();
        builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
        builder.terminate_with_return(vec![]);

        build_basic_foo_with_return(&mut builder, foo_id, Some(inline_type));

        let ssa = builder.finish();

        let (acir_functions, _, _) = ssa
            .into_acir(&Brillig::default())
            .expect("Should compile manually written SSA into ACIR");
        // The expected result should look very similar to the above test expect that the input witnesses of the `Call`
        // opcodes will be different. The changes can discerned from the checks below.

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo` and an assert");

        check_call_opcode(&main_opcodes[0], 1, vec![Witness(0), Witness(1)], vec![Witness(2)]);
        // The output of the first call should be the input of the second call
        check_call_opcode(&main_opcodes[1], 1, vec![Witness(2), Witness(1)], vec![Witness(3)]);
    }

    fn basic_nested_call(inline_type: InlineType) {
        // SSA for the following Noir program:
        // fn main(x: Field, y: pub Field) {
        //     let z = func_with_nested_foo_call(x, y);
        //     let z2 = func_with_nested_foo_call(x, y);
        //     assert(z == z2);
        // }
        // #[fold]
        // fn func_with_nested_foo_call(x: Field, y: Field) -> Field {
        //     nested_call(x + 2, y)
        // }
        // #[fold]
        // fn foo(x: Field, y: Field) -> Field {
        //     assert(x != y);
        //     x
        // }
        //
        // SSA:
        // acir(inline) fn main f0 {
        //     b0(v0: Field, v1: Field):
        //       v3 = call f1(v0, v1)
        //       v4 = call f1(v0, v1)
        //       constrain v3 == v4
        //       return
        //     }
        // acir(fold) fn func_with_nested_foo_call f1 {
        //     b0(v0: Field, v1: Field):
        //       v3 = add v0, Field 2
        //       v5 = call f2(v3, v1)
        //       return v5
        //   }
        // acir(fold) fn foo f2 {
        //     b0(v0: Field, v1: Field):
        //       v2 = eq v0, v1
        //       constrain v2 == Field 0
        //       return v0
        //   }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::field());
        let main_v1 = builder.add_parameter(Type::field());

        let func_with_nested_foo_call_id = Id::test_new(1);
        let func_with_nested_foo_call = builder.import_function(func_with_nested_foo_call_id);
        let main_call1_results = builder
            .insert_call(func_with_nested_foo_call, vec![main_v0, main_v1], vec![Type::field()])
            .to_vec();
        let main_call2_results = builder
            .insert_call(func_with_nested_foo_call, vec![main_v0, main_v1], vec![Type::field()])
            .to_vec();
        builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
        builder.terminate_with_return(vec![]);

        builder.new_function(
            "func_with_nested_foo_call".into(),
            func_with_nested_foo_call_id,
            inline_type,
        );
        let func_with_nested_call_v0 = builder.add_parameter(Type::field());
        let func_with_nested_call_v1 = builder.add_parameter(Type::field());

        let two = builder.field_constant(2u128);
        let v0_plus_two = builder.insert_binary(func_with_nested_call_v0, BinaryOp::Add, two);

        let foo_id = Id::test_new(2);
        let foo_call = builder.import_function(foo_id);
        let foo_call = builder
            .insert_call(foo_call, vec![v0_plus_two, func_with_nested_call_v1], vec![Type::field()])
            .to_vec();
        builder.terminate_with_return(vec![foo_call[0]]);

        build_basic_foo_with_return(&mut builder, foo_id, Some(inline_type));

        let ssa = builder.finish();

        let (acir_functions, _, _) = ssa
            .into_acir(&Brillig::default())
            .expect("Should compile manually written SSA into ACIR");

        assert_eq!(acir_functions.len(), 3, "Should have three ACIR functions");

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo` and an assert");

        // Both of these should call func_with_nested_foo_call f1
        check_call_opcode(&main_opcodes[0], 1, vec![Witness(0), Witness(1)], vec![Witness(2)]);
        // The output of the first call should be the input of the second call
        check_call_opcode(&main_opcodes[1], 1, vec![Witness(0), Witness(1)], vec![Witness(3)]);

        let func_with_nested_call_acir = &acir_functions[1];
        let func_with_nested_call_opcodes = func_with_nested_call_acir.opcodes();

        assert_eq!(
            func_with_nested_call_opcodes.len(),
            3,
            "Should have an expression and a call to a nested `foo`"
        );
        // Should call foo f2
        check_call_opcode(
            &func_with_nested_call_opcodes[1],
            2,
            vec![Witness(2), Witness(1)],
            vec![Witness(3)],
        );
    }

    fn check_call_opcode(
        opcode: &Opcode,
        expected_id: u32,
        expected_inputs: Vec<Witness>,
        expected_outputs: Vec<Witness>,
    ) {
        match opcode {
            Opcode::Call { id, inputs, outputs, .. } => {
                assert_eq!(
                    *id, expected_id,
                    "Main was expected to call {expected_id} but got {}",
                    *id
                );
                for (expected_input, input) in expected_inputs.iter().zip(inputs) {
                    assert_eq!(
                        expected_input, input,
                        "Expected witness {expected_input:?} but got {input:?}"
                    );
                }
                for (expected_output, output) in expected_outputs.iter().zip(outputs) {
                    assert_eq!(
                        expected_output, output,
                        "Expected witness {expected_output:?} but got {output:?}"
                    );
                }
            }
            _ => panic!("Expected only Call opcode"),
        }
    }

    // Test that given multiple calls to the same brillig function we generate only one bytecode
    // and the appropriate Brillig call opcodes are generated
    #[test]
    fn multiple_brillig_calls_one_bytecode() {
        // acir(inline) fn main f0 {
        //     b0(v0: Field, v1: Field):
        //       v4 = call f1(v0, v1)
        //       v5 = call f1(v0, v1)
        //       v6 = call f1(v0, v1)
        //       v7 = call f2(v0, v1)
        //       v8 = call f1(v0, v1)
        //       v9 = call f2(v0, v1)
        //       return
        // }
        // brillig fn foo f1 {
        // b0(v0: Field, v1: Field):
        //     v2 = eq v0, v1
        //     constrain v2 == u1 0
        //     return v0
        // }
        // brillig fn foo f2 {
        //     b0(v0: Field, v1: Field):
        //       v2 = eq v0, v1
        //       constrain v2 == u1 0
        //       return v0
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::field());
        let main_v1 = builder.add_parameter(Type::field());

        let foo_id = Id::test_new(1);
        let foo = builder.import_function(foo_id);
        let bar_id = Id::test_new(2);
        let bar = builder.import_function(bar_id);

        // Insert multiple calls to the same Brillig function
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        // Interleave a call to a separate Brillig function to make sure that we can call multiple separate Brillig functions
        builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.terminate_with_return(vec![]);

        build_basic_foo_with_return(&mut builder, foo_id, None);
        build_basic_foo_with_return(&mut builder, bar_id, None);

        let ssa = builder.finish();
        let brillig = ssa.to_brillig(false);

        let (acir_functions, brillig_functions, _) =
            ssa.into_acir(&brillig).expect("Should compile manually written SSA into ACIR");

        assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
        assert_eq!(brillig_functions.len(), 2, "Should only have generated two Brillig functions");

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        assert_eq!(main_opcodes.len(), 6, "Should have four calls to f1 and two calls to f2");

        // We should only have `BrilligCall` opcodes in `main`
        for (i, opcode) in main_opcodes.iter().enumerate() {
            match opcode {
                Opcode::BrilligCall { id, .. } => {
                    let expected_id = if i == 3 || i == 5 { 1 } else { 0 };
                    assert_eq!(*id, expected_id, "Expected an id of {expected_id} but got {id}");
                }
                _ => panic!("Expected only Brillig call opcode"),
            }
        }
    }

    // Test that given multiple primitive operations that are represented by Brillig directives (e.g. invert/quotient),
    // we will only generate one bytecode and the appropriate Brillig call opcodes are generated.
    #[test]
    fn multiple_brillig_stdlib_calls() {
        // acir(inline) fn main f0 {
        //     b0(v0: u32, v1: u32, v2: u32):
        //       v3 = div v0, v1
        //       constrain v3 == v2
        //       v4 = div v1, v2
        //       constrain v4 == u32 1
        //       return
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::unsigned(32));
        let main_v1 = builder.add_parameter(Type::unsigned(32));
        let main_v2 = builder.add_parameter(Type::unsigned(32));

        // Call a primitive operation that uses Brillig
        let v0_div_v1 = builder.insert_binary(main_v0, BinaryOp::Div, main_v1);
        builder.insert_constrain(v0_div_v1, main_v2, None);

        // Call the same primitive operation again
        let v1_div_v2 = builder.insert_binary(main_v1, BinaryOp::Div, main_v2);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        builder.insert_constrain(v1_div_v2, one, None);

        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        println!("{}", ssa);

        // The Brillig bytecode we insert for the stdlib is hardcoded so we do not need to provide any
        // Brillig artifacts to the ACIR gen pass.
        let (acir_functions, brillig_functions, _) = ssa
            .into_acir(&Brillig::default())
            .expect("Should compile manually written SSA into ACIR");

        assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
        // We expect two brillig functions:
        //   - Quotient (shared between both divisions)
        //   - Inversion, caused by division-by-zero check (shared between both divisions)
        assert_eq!(brillig_functions.len(), 2, "Should only have generated two Brillig functions");

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        check_brillig_calls(
            &acir_functions[0].brillig_stdlib_func_locations,
            main_opcodes,
            0,
            4,
            0,
        );
    }

    // Test that given both hardcoded Brillig directives and calls to normal Brillig functions,
    // we generate a single bytecode for the directives and a single bytecode for the normal Brillig calls.
    #[test]
    fn brillig_stdlib_calls_with_regular_brillig_call() {
        // acir(inline) fn main f0 {
        //     b0(v0: u32, v1: u32, v2: u32):
        //       v4 = div v0, v1
        //       constrain v4 == v2
        //       v5 = call f1(v0, v1)
        //       v6 = call f1(v0, v1)
        //       v7 = div v1, v2
        //       constrain v7 == u32 1
        //       return
        // }
        // brillig fn foo f1 {
        //   b0(v0: Field, v1: Field):
        //     v2 = eq v0, v1
        //     constrain v2 == u1 0
        //     return v0
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::unsigned(32));
        let main_v1 = builder.add_parameter(Type::unsigned(32));
        let main_v2 = builder.add_parameter(Type::unsigned(32));

        let foo_id = Id::test_new(1);
        let foo = builder.import_function(foo_id);

        // Call a primitive operation that uses Brillig
        let v0_div_v1 = builder.insert_binary(main_v0, BinaryOp::Div, main_v1);
        builder.insert_constrain(v0_div_v1, main_v2, None);

        // Insert multiple calls to the same Brillig function
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();

        // Call the same primitive operation again
        let v1_div_v2 = builder.insert_binary(main_v1, BinaryOp::Div, main_v2);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        builder.insert_constrain(v1_div_v2, one, None);

        builder.terminate_with_return(vec![]);

        build_basic_foo_with_return(&mut builder, foo_id, None);

        let ssa = builder.finish();
        // We need to generate  Brillig artifacts for the regular Brillig function and pass them to the ACIR generation pass.
        let brillig = ssa.to_brillig(false);
        println!("{}", ssa);

        let (acir_functions, brillig_functions, _) =
            ssa.into_acir(&brillig).expect("Should compile manually written SSA into ACIR");

        assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
        // We expect 3 brillig functions:
        //   - Quotient (shared between both divisions)
        //   - Inversion, caused by division-by-zero check (shared between both divisions)
        //   - Custom brillig function `foo`
        assert_eq!(
            brillig_functions.len(),
            3,
            "Should only have generated three Brillig functions"
        );

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        check_brillig_calls(
            &acir_functions[0].brillig_stdlib_func_locations,
            main_opcodes,
            1,
            4,
            2,
        );
    }

    // Test that given both normal Brillig calls, Brillig stdlib calls, and non-inlined ACIR calls, that we accurately generate ACIR.
    #[test]
    fn brillig_stdlib_calls_with_multiple_acir_calls() {
        // acir(inline) fn main f0 {
        //     b0(v0: u32, v1: u32, v2: u32):
        //       v4 = div v0, v1
        //       constrain v4 == v2
        //       v5 = call f1(v0, v1)
        //       v6 = call f2(v0, v1)
        //       v7 = div v1, v2
        //       constrain v7 == u32 1
        //       return
        // }
        // brillig fn foo f1 {
        //   b0(v0: Field, v1: Field):
        //     v2 = eq v0, v1
        //     constrain v2 == u1 0
        //     return v0
        // }
        // acir(fold) fn foo f2 {
        //     b0(v0: Field, v1: Field):
        //       v2 = eq v0, v1
        //       constrain v2 == u1 0
        //       return v0
        //   }
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), foo_id);
        let main_v0 = builder.add_parameter(Type::unsigned(32));
        let main_v1 = builder.add_parameter(Type::unsigned(32));
        let main_v2 = builder.add_parameter(Type::unsigned(32));

        let foo_id = Id::test_new(1);
        let foo = builder.import_function(foo_id);
        let bar_id = Id::test_new(2);
        let bar = builder.import_function(bar_id);

        // Call a primitive operation that uses Brillig
        let v0_div_v1 = builder.insert_binary(main_v0, BinaryOp::Div, main_v1);
        builder.insert_constrain(v0_div_v1, main_v2, None);

        // Insert multiple calls to the same Brillig function
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
        builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();

        // Call the same primitive operation again
        let v1_div_v2 = builder.insert_binary(main_v1, BinaryOp::Div, main_v2);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        builder.insert_constrain(v1_div_v2, one, None);

        builder.terminate_with_return(vec![]);

        // Build a Brillig function
        build_basic_foo_with_return(&mut builder, foo_id, None);
        // Build an ACIR function which has the same logic as the Brillig function above
        build_basic_foo_with_return(&mut builder, bar_id, Some(InlineType::Fold));

        let ssa = builder.finish();
        // We need to generate  Brillig artifacts for the regular Brillig function and pass them to the ACIR generation pass.
        let brillig = ssa.to_brillig(false);
        println!("{}", ssa);

        let (acir_functions, brillig_functions, _) =
            ssa.into_acir(&brillig).expect("Should compile manually written SSA into ACIR");

        assert_eq!(acir_functions.len(), 2, "Should only have two ACIR functions");
        // We expect 3 brillig functions:
        //   - Quotient (shared between both divisions)
        //   - Inversion, caused by division-by-zero check (shared between both divisions)
        //   - Custom brillig function `foo`
        assert_eq!(
            brillig_functions.len(),
            3,
            "Should only have generated three Brillig functions"
        );

        let main_acir = &acir_functions[0];
        let main_opcodes = main_acir.opcodes();
        check_brillig_calls(
            &acir_functions[0].brillig_stdlib_func_locations,
            main_opcodes,
            1,
            4,
            2,
        );

        let foo_acir = &acir_functions[1];
        let foo_opcodes = foo_acir.opcodes();
        check_brillig_calls(&acir_functions[1].brillig_stdlib_func_locations, foo_opcodes, 1, 1, 0);
    }

    fn check_brillig_calls(
        brillig_stdlib_function_locations: &BTreeMap<OpcodeLocation, BrilligStdlibFunc>,
        opcodes: &[Opcode],
        num_normal_brillig_functions: u32,
        expected_num_stdlib_calls: u32,
        expected_num_normal_calls: u32,
    ) {
        // First we check calls to the Brillig stdlib
        let mut num_brillig_stdlib_calls = 0;
        for (i, (opcode_location, brillig_stdlib_func)) in
            brillig_stdlib_function_locations.iter().enumerate()
        {
            // We can take just modulo 2 to determine the expected ID as we only code generated two Brillig stdlib function
            let stdlib_func_index = (i % 2) as u32;
            if stdlib_func_index == 0 {
                assert!(matches!(brillig_stdlib_func, BrilligStdlibFunc::Inverse));
            } else {
                assert!(matches!(brillig_stdlib_func, BrilligStdlibFunc::Quotient(_)));
            }

            match opcode_location {
                OpcodeLocation::Acir(acir_index) => {
                    match opcodes[*acir_index] {
                        Opcode::BrilligCall { id, .. } => {
                            // Brillig stdlib function calls are only resolved at the end of ACIR generation so their
                            // IDs are expected to always reference Brillig bytecode at the end of the Brillig functions list.
                            // We have one normal Brillig call so we add one here to the std lib function's index within the std lib.
                            let expected_id = stdlib_func_index + num_normal_brillig_functions;
                            assert_eq!(id, expected_id, "Expected {expected_id} but got {id}");
                            num_brillig_stdlib_calls += 1;
                        }
                        _ => panic!("Expected BrilligCall opcode"),
                    }
                }
                _ => panic!("Expected OpcodeLocation::Acir"),
            }
        }

        assert_eq!(
            num_brillig_stdlib_calls, expected_num_stdlib_calls,
            "Should have {expected_num_stdlib_calls} BrilligCall opcodes to stdlib functions but got {num_brillig_stdlib_calls}"
        );

        // Check the normal Brillig calls
        // This check right now expects to only call one Brillig function.
        let mut num_normal_brillig_calls = 0;
        for (i, opcode) in opcodes.iter().enumerate() {
            if let Opcode::BrilligCall { id, .. } = opcode {
                if brillig_stdlib_function_locations.get(&OpcodeLocation::Acir(i)).is_some() {
                    // We should have already checked Brillig stdlib functions and only want to check normal Brillig calls here
                    continue;
                }
                // We only generate one normal Brillig call so we should expect a function ID of `0`
                let expected_id = 0u32;
                assert_eq!(*id, expected_id, "Expected an id of {expected_id} but got {id}");
                num_normal_brillig_calls += 1;
            }
        }

        assert_eq!(
            num_normal_brillig_calls, expected_num_normal_calls,
            "Should have {expected_num_normal_calls} BrilligCall opcodes to normal Brillig functions but got {num_normal_brillig_calls}"
        );
    }
}
