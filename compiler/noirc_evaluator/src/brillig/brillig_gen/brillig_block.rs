//! Module containing Brillig-gen logic specific to an SSA function's basic blocks.
use crate::brillig::brillig_ir::artifact::Label;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
};

use crate::brillig::brillig_ir::registers::{Allocated, RegisterAllocator};
use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};
use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    function::FunctionId,
    instruction::{Instruction, InstructionId, TerminatorInstruction},
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::{FieldElement, acir::AcirField};
use iter_extended::vecmap;
use noirc_errors::call_stack::{CallStackHelper, CallStackId};
use num_bigint::BigUint;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::BTreeSet;

use super::brillig_block_variables::{BlockVariables, allocate_value_with_type};
use super::brillig_fn::FunctionContext;
use super::brillig_globals::HoistedConstantsToBrilligGlobals;
use super::constant_allocation::InstructionLocation;

/// Context structure for compiling a [function block][crate::ssa::ir::basic_block::BasicBlock] into Brillig bytecode.
pub(crate) struct BrilligBlock<'block, Registers: RegisterAllocator> {
    /// Per-function context shared across all of a function's blocks
    pub(crate) function_context: &'block mut FunctionContext,
    /// The basic block that is being converted
    pub(crate) block_id: BasicBlockId,
    /// Context for creating brillig opcodes
    pub(crate) brillig_context: &'block mut BrilligContext<FieldElement, Registers>,
    /// Tracks the available variable during the codegen of the block
    pub(crate) variables: BlockVariables,
    /// For each instruction, the set of values that are not used anymore after it.
    pub(crate) last_uses: HashMap<InstructionId, HashSet<ValueId>>,

    /// Mapping of SSA [ValueId]s to their already instantiated values in the Brillig IR.
    pub(crate) globals: &'block HashMap<ValueId, BrilligVariable>,
    /// Pre-instantiated constants values shared across functions which have hoisted to the global memory space.
    pub(crate) hoisted_global_constants: &'block HoistedConstantsToBrilligGlobals,
    /// Status variable for whether we are generating Brillig bytecode for a function or globals.
    /// This is primarily used for gating local variable specific logic.
    /// For example, liveness analysis for globals is unnecessary (and adds complexity),
    /// and instead globals live throughout the entirety of the program.
    pub(crate) building_globals: bool,
}

impl<'block, Registers: RegisterAllocator> BrilligBlock<'block, Registers> {
    /// Converts an SSA basic block into a sequence of Brillig opcodes.
    ///
    /// This method contains the necessary initial variable and register setup for compiling
    /// an SSA block by accessing the pre-computed liveness context.
    pub(crate) fn compile_block(
        function_context: &'block mut FunctionContext,
        brillig_context: &'block mut BrilligContext<FieldElement, Registers>,
        block_id: BasicBlockId,
        dfg: &DataFlowGraph,
        call_stacks: &mut CallStackHelper,
        globals: &'block HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &'block HoistedConstantsToBrilligGlobals,
    ) {
        let live_in = function_context.liveness.get_live_in(&block_id);

        let mut live_in_no_globals = HashSet::default();
        for value in live_in {
            if let Value::NumericConstant { constant, typ } = dfg[*value] {
                if hoisted_global_constants.contains_key(&(constant, typ)) {
                    continue;
                }
            }
            if !dfg.is_global(*value) {
                live_in_no_globals.insert(*value);
            }
        }

        let variables = BlockVariables::new(live_in_no_globals);

        // Replace the previous registers with a new instance, where the currently live variables are pre-allocated.
        // These might be deallocated and reused if their last use in this block indicates they are dead,
        // but then become pre-allocated in a new block again, depending on the order of processing.
        brillig_context.set_allocated_registers(
            variables
                .get_available_variable_allocations(function_context)
                .into_iter()
                .map(|variable| variable.extract_register())
                .collect(),
        );
        let last_uses = function_context.liveness.get_last_uses(&block_id).clone();

        let mut brillig_block = BrilligBlock {
            function_context,
            block_id,
            brillig_context,
            variables,
            last_uses,
            globals,
            hoisted_global_constants,
            building_globals: false,
        };

        brillig_block.convert_block(dfg, call_stacks);
    }

    /// Converts SSA globals into Brillig global values.
    ///
    /// Global values can be:
    /// - Numeric constants
    /// - Instructions that compute global values
    /// - Pre-hoisted constants (shared across functions and stored in global memory)
    ///
    /// This method expects SSA globals to already be converted to a [DataFlowGraph]
    /// as to share codegen logic with standard SSA function blocks.
    ///
    /// This method also emits any necessary debugging initialization logic (e.g., allocating a counter used
    /// to track array copies).
    ///
    /// # Returns
    /// A map of hoisted (constant, type) pairs to their allocated Brillig variables,
    /// which are used to resolve references to these constants throughout Brillig lowering.
    ///
    /// # Panics
    /// - Globals graph contains values other than a [constant][Value::NumericConstant] or [instruction][Value::Instruction]
    pub(crate) fn compile_globals(
        &mut self,
        globals: &DataFlowGraph,
        used_globals: &HashSet<ValueId>,
        call_stacks: &mut CallStackHelper,
        hoisted_global_constants: &BTreeSet<(FieldElement, NumericType)>,
    ) -> HashMap<(FieldElement, NumericType), Allocated<BrilligVariable, Registers>> {
        // Using the end of the global memory space adds more complexity as we
        // have to account for possible register de-allocations as part of regular global compilation.
        // Thus, we want to allocate any reserved global slots first.

        // If we want to print the array copy count in the end, we reserve teh 0 slot.
        if self.brillig_context.count_array_copies() {
            // Detach from the register so it's never deallocated.
            let new_variable =
                allocate_value_with_type(self.brillig_context, Type::unsigned(32)).detach();
            self.brillig_context
                .const_instruction(new_variable.extract_single_addr(), FieldElement::zero());
        };

        for (id, value) in globals.values_iter() {
            if !used_globals.contains(&id) {
                continue;
            }
            match value {
                Value::NumericConstant { .. } => {
                    self.convert_ssa_value(id, globals);
                }
                Value::Instruction { instruction, .. } => {
                    self.convert_ssa_instruction(*instruction, globals, call_stacks);
                }
                _ => {
                    panic!(
                        "Expected either an instruction or a numeric constant for a global value"
                    )
                }
            }
        }

        // Allocate and initialize hoisted constants. These don't have a variable ID associated with them,
        // so we return them explicitly, while the allocated global variables are in the `ssa_value_allocations`
        // field of the `FunctionContext`.
        let mut new_hoisted_constants = HashMap::default();
        for (constant, typ) in hoisted_global_constants.iter().copied() {
            let new_variable = allocate_value_with_type(self.brillig_context, Type::Numeric(typ));
            self.brillig_context.const_instruction(new_variable.extract_single_addr(), constant);
            if new_hoisted_constants.insert((constant, typ), new_variable).is_some() {
                unreachable!("ICE: ({constant:?}, {typ:?}) was already in cache");
            }
        }

        new_hoisted_constants
    }

    /// Internal method for [BrilligBlock::compile_block] that actually kicks off the Brillig compilation process.
    ///
    /// At this point any Brillig context should be contained in [BrilligBlock], and this function should
    /// only need to accept external SSA and debugging structures.
    fn convert_block(&mut self, dfg: &DataFlowGraph, call_stacks: &mut CallStackHelper) {
        // Add a label for this block
        let block_label = self.create_block_label_for_current_function(self.block_id);
        self.brillig_context.enter_context(block_label);

        // Allocate variables for parameter passing between blocks.
        self.convert_block_params(dfg);

        let block = &dfg[self.block_id];

        // Convert all of the instructions into the block.
        for instruction_id in block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg, call_stacks);
        }

        // Process the block's terminator instruction.
        let terminator_instruction =
            block.terminator().expect("block is expected to be constructed");

        // If we are exiting the entry point, we may want to print the array copy count, for debug purposes.
        if self.brillig_context.count_array_copies()
            && matches!(terminator_instruction, TerminatorInstruction::Return { .. })
            && self.function_context.is_entry_point
        {
            self.brillig_context.emit_println_of_array_copy_counter();
        }

        self.convert_ssa_terminator(terminator_instruction, dfg);
    }

    /// Creates a unique global label for a block.
    ///
    /// This uses the current function's function ID and the block ID,
    /// making the assumption that the block ID passed in belongs to this function.
    fn create_block_label_for_current_function(&self, block_id: BasicBlockId) -> Label {
        Self::create_block_label(self.function_context.function_id(), block_id)
    }
    /// Creates a unique label for a block using the function Id and the block ID.
    ///
    /// We implicitly assume that the function ID and the block ID is enough
    /// for us to create a unique label across functions and blocks.
    ///
    /// This is so that during linking there are no duplicates or labels being overwritten.
    fn create_block_label(function_id: FunctionId, block_id: BasicBlockId) -> Label {
        Label::block(function_id, block_id)
    }

    /// Converts an SSA terminator instruction into the necessary opcodes:
    /// * allocates the hoisted constants which are used by dominated blocks
    /// * for jumps:
    ///   * copies the arguments to the registers allocated in [Self::convert_block_params]
    ///   * adds jump opcodes to the labels of the destination blocks
    /// * for return it allocates registers for the return values and copies from variables.
    fn convert_ssa_terminator(
        &mut self,
        terminator_instruction: &TerminatorInstruction,
        dfg: &DataFlowGraph,
    ) {
        self.initialize_constants(dfg, InstructionLocation::Terminator);

        match terminator_instruction {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack: _,
            } => {
                let condition = self.convert_ssa_single_addr_value(*condition, dfg);
                self.brillig_context.jump_if_instruction(
                    condition.address,
                    self.create_block_label_for_current_function(*then_destination),
                );
                self.brillig_context.jump_instruction(
                    self.create_block_label_for_current_function(*else_destination),
                );
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                let destination_block = &dfg[*destination];
                for (arg, param) in arguments.iter().zip(destination_block.parameters()) {
                    // Destinations are block parameters, so they should have been allocated previously in `create_block_params`.
                    let param = self.variables.get_allocation(self.function_context, *param);
                    let arg = self.convert_ssa_value(*arg, dfg);
                    self.brillig_context
                        .mov_instruction(param.extract_register(), arg.extract_register());
                }
                self.brillig_context
                    .jump_instruction(self.create_block_label_for_current_function(*destination));
            }
            TerminatorInstruction::Return { return_values, .. } => {
                let return_registers = vecmap(return_values, |value_id| {
                    // Get the allocations of the values to be returned.
                    self.convert_ssa_value(*value_id, dfg)
                });
                self.brillig_context.codegen_return(&return_registers);
            }
            TerminatorInstruction::Unreachable { .. } => {
                // If we assume this is unreachable code then there's nothing to do here
            }
        }
    }

    /// Allocates the block parameters that the given block is defining.
    ///
    /// We don't allocate the block parameters of the block itself here, we allocate the parameters the block is defining
    /// for the descendant blocks it immediately dominates. Since predecessors to a block have to know where the parameters
    /// of the block are allocated to pass data to it in [Self::convert_ssa_terminator], the block parameters need to be
    /// defined/allocated before the given block. [VariableLiveness](crate::brillig::brillig_gen::variable_liveness::VariableLiveness)
    /// decides when the block parameters are defined.
    ///
    /// For the entry block, the defined block params will be the params of the function + any extra params of blocks it's the immediate dominator of.
    fn convert_block_params(&mut self, dfg: &DataFlowGraph) {
        for param_id in self.function_context.liveness.defined_block_params(&self.block_id) {
            let value = &dfg[param_id];
            let Value::Param { typ: param_type, .. } = value else {
                unreachable!("ICE: Only Param type values should appear in block parameters");
            };
            match param_type {
                Type::Numeric(_) | Type::Array(..) | Type::Vector(..) | Type::Reference(_) => {
                    // Simple parameters and arrays are passed as already filled registers.
                    // In the case of arrays, the values should already be in memory and the register should be a valid pointer to the array.
                    // For vectors, two registers are passed, the pointer to the data and a register holding the size of the vector.
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        param_id,
                        dfg,
                    );
                }
                Type::Function => unreachable!(
                    "ICE: Type::Function Param not supported; should have been removed by defunctionalization."
                ),
            }
        }
    }

    /// Converts an SSA instruction into a sequence of Brillig opcodes.
    ///
    /// If this is the last time a variable is used in this block, its memory slot gets deallocated, unless it's a global.
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        call_stacks: &mut CallStackHelper,
    ) {
        let instruction = &dfg[instruction_id];
        let call_stack = dfg.get_instruction_call_stack(instruction_id);
        let call_stack_new_id = call_stacks.get_or_insert_locations(&call_stack);
        self.brillig_context.set_call_stack(call_stack_new_id);

        self.initialize_constants(dfg, InstructionLocation::Instruction(instruction_id));

        match instruction {
            Instruction::Binary(binary) => {
                self.codegen_binary(instruction_id, binary, dfg);
            }
            Instruction::Constrain(lhs, rhs, assert_message) => {
                self.codegen_constrain(*lhs, *rhs, assert_message, dfg);
            }
            Instruction::ConstrainNotEqual(..) => {
                unreachable!("only implemented in ACIR")
            }
            Instruction::Allocate => {
                self.codegen_allocate(instruction_id, dfg);
            }
            Instruction::Store { address, value } => {
                self.codegen_store(*address, *value, dfg);
            }
            Instruction::Load { address } => {
                self.codegen_load(instruction_id, *address, dfg);
            }
            Instruction::Not(value) => {
                self.codegen_not(instruction_id, *value, dfg);
            }
            Instruction::Call { func, arguments } => {
                self.codegen_call(instruction_id, *func, arguments, dfg);
            }
            Instruction::Truncate { value, bit_size, .. } => {
                self.codegen_truncate(instruction_id, *value, *bit_size, dfg);
            }
            Instruction::Cast(value, _) => {
                self.codegen_cast(instruction_id, *value, dfg);
            }
            Instruction::ArrayGet { array, index } => {
                self.codegen_array_get(instruction_id, *array, *index, dfg);
            }
            Instruction::ArraySet { array, index, value, mutable } => {
                self.codegen_array_set(instruction_id, *array, *index, *value, *mutable, dfg);
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                self.codegen_range_check(*value, *max_bit_size, assert_message.as_ref(), dfg);
            }
            Instruction::IncrementRc { value } => {
                self.codegen_increment_rc(*value, dfg);
            }
            Instruction::DecrementRc { value } => {
                self.codegen_decrement_rc(*value, dfg);
            }
            Instruction::EnableSideEffectsIf { .. } => {
                unreachable!("enable_side_effects not supported by brillig")
            }
            Instruction::IfElse { then_condition, then_value, else_condition: _, else_value } => {
                // The `else_condition` is assumed to be the opposite of the `then_condition`.
                self.codegen_if_else(
                    instruction_id,
                    *then_condition,
                    *then_value,
                    *else_value,
                    dfg,
                );
            }
            Instruction::MakeArray { elements: array, typ } => {
                self.codegen_make_array(instruction_id, array, typ, dfg);
            }
            Instruction::Noop => (),
        };

        if !self.building_globals {
            let dead_variables = self
                .last_uses
                .get(&instruction_id)
                .expect("Last uses for instruction should have been computed");

            for dead_variable in dead_variables {
                // Globals are reserved throughout the entirety of the program
                let is_global = dfg.is_global(*dead_variable);
                let is_hoisted_global = self.get_hoisted_global(dfg, *dead_variable).is_some();
                if !is_global && !is_hoisted_global {
                    self.variables.remove_variable(
                        dead_variable,
                        self.function_context,
                        self.brillig_context,
                    );
                }
            }
        }

        // Clear the call stack; it only applied to this instruction.
        self.brillig_context.set_call_stack(CallStackId::root());
    }

    /// Converts an SSA cast to a sequence of Brillig opcodes.
    /// Casting is only necessary when shrinking the bit size of a numeric value.
    fn convert_cast(&mut self, destination: SingleAddrVariable, source: SingleAddrVariable) {
        // We assume that `source` is a valid `target_type` as it's expected that a truncate instruction was emitted
        // to ensure this is the case.

        self.brillig_context.cast_instruction(destination, source);
    }

    /// Initializes constants allocated to a [InstructionLocation] by [ConstantAllocation](crate::brillig::brillig_gen::constant_allocation::ConstantAllocation).
    ///
    /// It is expected that this method is called before converting an SSA instruction to Brillig
    /// and the constants to be initialized have been precomputed and stored in [FunctionContext::constant_allocation].
    fn initialize_constants(&mut self, dfg: &DataFlowGraph, location: InstructionLocation) {
        let Some(constants) = self
            .function_context
            .constant_allocation
            .allocated_at_location(self.block_id, location)
            .map(|c| c.to_vec())
        else {
            return;
        };

        for constant_id in constants {
            self.convert_ssa_value(constant_id, dfg);
        }
    }

    /// Converts an SSA [ValueId] into a [BrilligVariable]. Initializes if necessary, or returns an existing allocation.
    ///
    /// This method also first checks whether the SSA value is a hoisted global constant.
    /// If it has already been initialized in the global space, we return the already existing variable.
    /// If an SSA value is a [Value::Global], we check whether the value exists in the [BrilligBlock::globals] map,
    /// otherwise the method panics.
    pub(crate) fn convert_ssa_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let value = &dfg[value_id];

        if let Some(variable) = self.get_hoisted_global(dfg, value_id) {
            return variable;
        }

        match value {
            Value::Global(_) => {
                unreachable!("Expected global value to be resolve to its inner value");
            }
            Value::Param { .. } | Value::Instruction { .. } => {
                // All block parameters and instruction results should have already been
                // converted to registers so we fetch from the cache.
                if dfg.is_global(value_id) {
                    *self.globals.get(&value_id).unwrap_or_else(|| {
                        panic!("ICE: Global value not found in cache {value_id}")
                    })
                } else {
                    self.variables.get_allocation(self.function_context, value_id)
                }
            }
            Value::NumericConstant { constant, .. } => {
                // Constants might have been converted previously or not, so we get or create and
                // (re)initialize the value inside.
                if self.variables.is_allocated(&value_id) {
                    self.variables.get_allocation(self.function_context, value_id)
                } else if dfg.is_global(value_id) {
                    *self.globals.get(&value_id).unwrap_or_else(|| {
                        panic!("ICE: Global value not found in cache {value_id}")
                    })
                } else {
                    let new_variable = self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        value_id,
                        dfg,
                    );

                    self.brillig_context
                        .const_instruction(new_variable.extract_single_addr(), *constant);
                    new_variable
                }
            }
            Value::Function(_) => {
                // For the debugger instrumentation we want to allow passing
                // around values representing function pointers, even though
                // there is no interaction with the function possible given that
                // value.
                let new_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    value_id,
                    dfg,
                );

                self.brillig_context.const_instruction(
                    new_variable.extract_single_addr(),
                    value_id.to_u32().into(),
                );
                new_variable
            }
            Value::Intrinsic(_) | Value::ForeignFunction(_) => {
                unreachable!("ICE: Cannot convert value to Brillig: {value:?}")
            }
        }
    }

    /// Converts an SSA `ValueId` into a single `MemoryAddress`. Initializes if necessary.
    ///
    /// Panics if the value was converted to an array or vector.
    pub(crate) fn convert_ssa_single_addr_value(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> SingleAddrVariable {
        let variable = self.convert_ssa_value(value_id, dfg);
        variable.extract_single_addr()
    }

    /// If the supplied value is a numeric constant check whether it is exists within
    /// the precomputed [hoisted globals map][Self::hoisted_global_constants].
    /// If the variable exists as a hoisted global return that value, otherwise return `None`.
    fn get_hoisted_global(
        &self,
        dfg: &DataFlowGraph,
        value_id: ValueId,
    ) -> Option<BrilligVariable> {
        if let Value::NumericConstant { constant, typ } = &dfg[value_id] {
            if let Some(variable) = self.hoisted_global_constants.get(&(*constant, *typ)) {
                return Some(*variable);
            }
        }
        None
    }

    /// Define the result variable, convert the value, then generate Brillig opcodes to negate the variable.
    fn codegen_not(&mut self, instruction_id: InstructionId, value: ValueId, dfg: &DataFlowGraph) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let condition_register = self.convert_ssa_single_addr_value(value, dfg);
        let result_register = self.variables.define_single_addr_variable(
            self.function_context,
            self.brillig_context,
            result_id,
            dfg,
        );
        self.brillig_context.not_instruction(condition_register, result_register);
    }

    /// Define the result variable, convert the value, then generate Brillig opcodes to truncate the variable.
    fn codegen_truncate(
        &mut self,
        instruction_id: InstructionId,
        value: ValueId,
        bit_size: u32,
        dfg: &DataFlowGraph,
    ) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let destination_register = self.variables.define_single_addr_variable(
            self.function_context,
            self.brillig_context,
            result_id,
            dfg,
        );
        let source_register = self.convert_ssa_single_addr_value(value, dfg);
        self.brillig_context.codegen_truncate(destination_register, source_register, bit_size);
    }

    /// Define the result variable, convert the value, then generate Brillig opcodes to truncate the variable.
    fn codegen_cast(&mut self, instruction_id: InstructionId, value: ValueId, dfg: &DataFlowGraph) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let destination_variable = self.variables.define_single_addr_variable(
            self.function_context,
            self.brillig_context,
            result_id,
            dfg,
        );
        let source_variable = self.convert_ssa_single_addr_value(value, dfg);
        self.convert_cast(destination_variable, source_variable);
    }

    /// Convert the value, and if its bit size is larger than the maximum bit size,
    /// then generate opcodes to constrain the value to be no greater than the maximum
    /// value as a `Field`.
    fn codegen_range_check(
        &mut self,
        value: ValueId,
        max_bit_size: u32,
        assert_message: Option<&String>,
        dfg: &DataFlowGraph,
    ) {
        let value = self.convert_ssa_single_addr_value(value, dfg);
        // SSA generates redundant range checks. A range check with a max bit size >= value.bit_size will always pass.
        if value.bit_size > max_bit_size {
            // Cast original value to field
            let left = self.brillig_context.allocate_single_addr(FieldElement::max_num_bits());
            self.convert_cast(*left, value);

            // Create a field constant with the max
            let max = BigUint::from(2_u128).pow(max_bit_size) - BigUint::from(1_u128);
            let right = self.brillig_context.make_constant_instruction(
                FieldElement::from_be_bytes_reduce(&max.to_bytes_be()),
                FieldElement::max_num_bits(),
            );

            // Check if lte max
            let condition = self.brillig_context.allocate_single_addr_bool();
            self.brillig_context.binary_instruction(
                *left,
                *right,
                *condition,
                BrilligBinaryOp::LessThanEquals,
            );

            self.brillig_context.codegen_constrain(*condition, assert_message.cloned());
        }
    }

    /// Convert the condition and the values, define a variable for the result,
    /// then generate opcodes for a conditional move.
    ///
    /// Panics if the `then_value` and `else_value` are incompatible.
    fn codegen_if_else(
        &mut self,
        instruction_id: InstructionId,
        then_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
        dfg: &DataFlowGraph,
    ) {
        let then_condition = self.convert_ssa_single_addr_value(then_condition, dfg);
        let then_value = self.convert_ssa_value(then_value, dfg);
        let else_value = self.convert_ssa_value(else_value, dfg);

        let [result_id] = dfg.instruction_result(instruction_id);
        let result = self.variables.define_variable(
            self.function_context,
            self.brillig_context,
            result_id,
            dfg,
        );

        match (then_value, else_value) {
            (
                BrilligVariable::SingleAddr(then_address),
                BrilligVariable::SingleAddr(else_address),
            ) => {
                self.brillig_context.conditional_move_instruction(
                    then_condition,
                    then_address,
                    else_address,
                    result.extract_single_addr(),
                );
            }
            (
                BrilligVariable::BrilligArray(then_array),
                BrilligVariable::BrilligArray(else_array),
            ) => {
                // Pointer to the array which result from the if-else
                let pointer = self.brillig_context.allocate_register();
                self.brillig_context.conditional_move_instruction(
                    then_condition,
                    SingleAddrVariable::new_usize(then_array.pointer),
                    SingleAddrVariable::new_usize(else_array.pointer),
                    SingleAddrVariable::new_usize(*pointer),
                );
                let if_else_array = BrilligArray { pointer: *pointer, size: then_array.size };
                // Copy the if-else array to the result
                self.brillig_context
                    .call_array_copy_procedure(if_else_array, result.extract_array());
            }
            (
                BrilligVariable::BrilligVector(then_vector),
                BrilligVariable::BrilligVector(else_vector),
            ) => {
                // Pointer to the vector which result from the if-else
                let pointer = self.brillig_context.allocate_register();
                self.brillig_context.conditional_move_instruction(
                    then_condition,
                    SingleAddrVariable::new_usize(then_vector.pointer),
                    SingleAddrVariable::new_usize(else_vector.pointer),
                    SingleAddrVariable::new_usize(*pointer),
                );
                let if_else_vector = BrilligVector { pointer: *pointer };
                // Copy the if-else vector to the result
                self.brillig_context
                    .call_vector_copy_procedure(if_else_vector, result.extract_vector());
            }
            _ => unreachable!("ICE - then and else values must have the same type"),
        }
    }
}

/// Returns the type of the operation considering the types of the operands.
pub(crate) fn type_of_binary_operation(lhs_type: &Type, rhs_type: &Type) -> Type {
    match (lhs_type, rhs_type) {
        (_, Type::Function) | (Type::Function, _) => {
            unreachable!("Functions are invalid in binary operations")
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
        (Type::Numeric(lhs_type), Type::Numeric(rhs_type)) => {
            // If both sides are numeric type, then we expect their types to be the same.
            // For SHL and SHR they are also expected to be the same, but the SHR value itself cannot exceed the bit size.
            assert_eq!(
                lhs_type, rhs_type,
                "lhs and rhs types in a binary operation are always the same but got {lhs_type} and {rhs_type}"
            );
            Type::Numeric(*lhs_type)
        }
    }
}
