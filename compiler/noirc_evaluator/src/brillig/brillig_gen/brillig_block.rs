//! Module containing Brillig-gen logic specific to an SSA function's basic blocks.
use crate::brillig::brillig_ir::artifact::Label;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
};

use crate::brillig::brillig_ir::registers::{Allocated, RegisterAllocator};
use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext, ReservedRegisters};
use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    function::FunctionId,
    instruction::{Instruction, InstructionId, TerminatorInstruction},
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::{FieldElement, acir::AcirField, acir::brillig::MemoryAddress};
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

        let mut live_in_no_globals = live_in_no_globals(live_in, dfg, hoisted_global_constants);

        // Filter out spilled values — they don't have valid registers and must not be pre-allocated.
        // Reset the LRU with non-spilled live-in values for this block.
        if let Some(sm) = function_context.spill_manager.as_mut() {
            // Re-mark eagerly-spilled successor params so they are always reloaded
            // from the spill slot, regardless of what happened in previous blocks.
            sm.restore_permanent_spills();
            live_in_no_globals.retain(|v| !sm.is_spilled(v));
            sm.reset_lru_for_block(live_in_no_globals.iter().copied());
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

        // If we want to print the array copy count in the end, we reserve the 0 slot.
        if self.brillig_context.count_array_copies() {
            // Detach from the register so it's never deallocated.
            let new_variable =
                allocate_value_with_type(self.brillig_context, Type::unsigned(32)).detach();
            self.brillig_context
                .const_instruction(new_variable.extract_single_addr(), FieldElement::zero());
        }

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

    /// Check if allocating `n` more registers would exceed the stack frame limit.
    fn needs_spill_for(&self, n: usize) -> bool {
        if self.building_globals || self.function_context.spill_manager.is_none() {
            return false;
        }
        self.brillig_context.registers().available_registers() < n
    }

    /// Ensure there is capacity for `n` more register allocations by spilling if necessary.
    pub(crate) fn ensure_register_capacity(&mut self, n: usize) {
        while self.needs_spill_for(n) {
            self.spill_lru_value();
        }
    }

    /// Spill the least-recently-used value to the spill region
    fn spill_lru_value(&mut self) {
        // Extract data from spill_manager first (borrow checker prevent mutably borrowing the spill manager
        // while also accessing ssa_value_allocations immutably)
        let sm = self.function_context.spill_manager.as_mut().unwrap();
        let victim_id = sm.lru_victim().expect("No values available to spill");
        let offset = sm.allocate_spill_offset();

        let victim_var = *self.function_context.ssa_value_allocations.get(&victim_id).unwrap();
        let victim_reg = victim_var.extract_register();

        // Track that this function needs a spill region
        self.function_context.did_spill = true;
        self.function_context.max_spill_offset =
            self.function_context.max_spill_offset.max(offset + 1);
        let (scratch_addr, scratch_offset) = ReservedRegisters::spill_scratch();

        // 4-instruction spill:
        //   mov  scratch_addr, sp[1]          -- load spill base from stack slot
        //   const scratch_offset, offset      -- spill offset
        //   add  scratch_addr, scratch_addr, scratch_offset  -- compute absolute address
        //   store [scratch_addr], victim_reg  -- store value
        self.brillig_context.mov_instruction(scratch_addr, ReservedRegisters::spill_base_pointer());
        self.brillig_context
            .const_instruction(SingleAddrVariable::new_usize(scratch_offset), offset.into());
        self.brillig_context.memory_op_instruction(
            scratch_addr,
            scratch_offset,
            scratch_addr,
            BrilligBinaryOp::Add,
        );
        self.brillig_context.store_instruction(scratch_addr, victim_reg);

        // Free the victim's register so it can be reused
        self.brillig_context.deallocate_register(victim_reg);

        // Record the spill
        let sm = self.function_context.spill_manager.as_mut().unwrap();
        sm.record_spill(victim_id, offset, victim_var);
    }

    /// Reload a previously spilled value into a freshly allocated register
    fn reload_spilled_value(&mut self, value_id: ValueId) -> BrilligVariable {
        // Ensure capacity for the reload register (may trigger another spill)
        self.ensure_register_capacity(1);

        let sm = self.function_context.spill_manager.as_ref().unwrap();
        let spill_info = *sm.get_spill(&value_id).unwrap();
        let (scratch_addr, scratch_offset) = ReservedRegisters::spill_scratch();

        let new_reg = self.brillig_context.allocate_register().detach();

        // 4-instruction reload:
        //   mov  scratch_addr, sp[1]          -- load spill base from stack slot
        //   const scratch_offset, offset      -- spill offset
        //   add  scratch_addr, scratch_addr, scratch_offset  -- compute absolute address
        //   load new_reg, [scratch_addr]      -- load value
        self.brillig_context.mov_instruction(scratch_addr, ReservedRegisters::spill_base_pointer());
        self.brillig_context.const_instruction(
            SingleAddrVariable::new_usize(scratch_offset),
            spill_info.offset.into(),
        );
        self.brillig_context.memory_op_instruction(
            scratch_addr,
            scratch_offset,
            scratch_addr,
            BrilligBinaryOp::Add,
        );
        self.brillig_context.load_instruction(new_reg, scratch_addr);

        // Create updated variable with new register
        let new_var = spill_info.variable.with_register(new_reg);

        // Update SSA mapping to point to new register
        self.function_context.ssa_value_allocations.insert(value_id, new_var);

        // Unmark spilled (don't free the slot — loop iterations may re-execute this reload)
        let sm = self.function_context.spill_manager.as_mut().unwrap();
        sm.unmark_spilled(&value_id);
        sm.touch(value_id);

        // Re-add to available variables (was removed during spill)
        self.variables.add_available(value_id);

        new_var
    }

    /// Store non-param live-in values of `destination` into permanent spill
    /// slots. Uses only scratch registers (no register pressure concern).
    /// The destination's block will see them as spilled via
    /// the spiller manager and is responsible for emitting reload code.
    fn spill_non_param_live_ins(&mut self, destination: BasicBlockId, dfg: &DataFlowGraph) {
        if self.function_context.spill_manager.is_none() {
            return;
        }

        let dest_params: HashSet<_> = dfg[destination].parameters().iter().copied().collect();
        let live_in = self.function_context.liveness.get_live_in(&destination);
        let live_in_no_globals = live_in_no_globals(live_in, dfg, self.hoisted_global_constants);

        for value_id in live_in_no_globals {
            if dest_params.contains(&value_id) {
                continue;
            }

            let sm = self.function_context.spill_manager.as_mut().unwrap();

            // Already permanently tracked and currently spilled — slot has correct data
            if sm.get_permanent_spill_offset(&value_id).is_some() && sm.is_spilled(&value_id) {
                continue;
            }

            // If currently spilled (register was freed/reused) but not yet permanent,
            // promote the existing spill slot to permanent. No store needed — data is
            // already in the slot.
            if sm.is_spilled(&value_id) {
                let existing = *sm.get_spill(&value_id).unwrap();
                sm.record_permanent_spill(value_id, existing.offset, existing.variable);
                continue;
            }

            let var = *self.function_context.ssa_value_allocations.get(&value_id).unwrap();

            let sm = self.function_context.spill_manager.as_mut().unwrap();
            if let Some(off) = sm.get_permanent_spill_offset(&value_id) {
                // Value already has a permanent spill slot from a previous block.
                // The slot contains valid data (SSA values are immutable), so we
                // just need to re-mark it as spilled. No store or register
                // deallocation needed — the register may already be freed if the
                // value was reloaded and then died in this block.
                sm.record_spill(value_id, off, var);
            } else {
                // First encounter: allocate a permanent slot and store the value.
                let off = sm.allocate_spill_offset();
                sm.record_permanent_spill(value_id, off, var);

                // 4-instruction store via scratch registers
                let source_reg = var.extract_register();
                let (scratch_addr, scratch_offset) = ReservedRegisters::spill_scratch();
                self.brillig_context
                    .mov_instruction(scratch_addr, ReservedRegisters::spill_base_pointer());
                self.brillig_context
                    .const_instruction(SingleAddrVariable::new_usize(scratch_offset), off.into());
                self.brillig_context.memory_op_instruction(
                    scratch_addr,
                    scratch_offset,
                    scratch_addr,
                    BrilligBinaryOp::Add,
                );
                self.brillig_context.store_instruction(scratch_addr, source_reg);

                // Free the register: the value is now safely in the spill slot.
                // Without this, the register stays allocated but the value is marked
                // as spilled — `lru_victim()` can't reclaim it, creating "phantom"
                // allocations that exhaust the register file.
                // If the value is needed later (e.g., as a Jmp argument),
                // `convert_ssa_value` will see it's spilled and reload on demand.
                self.brillig_context.deallocate_register(source_reg);

                self.function_context.did_spill = true;
                self.function_context.max_spill_offset =
                    self.function_context.max_spill_offset.max(off + 1);
            }
        }
    }

    /// Wrapper for [BlockVariables::define_variable] that ensures register capacity
    /// and tracks the new value in the LRU.
    pub(crate) fn define_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        self.ensure_register_capacity(1);
        let var = self.variables.define_variable(
            self.function_context,
            self.brillig_context,
            value_id,
            dfg,
        );
        if let Some(sm) = self.function_context.spill_manager.as_mut() {
            sm.touch(value_id);
        }
        var
    }

    /// Defines a variable that fits in a single register and returns the allocated register.
    pub(crate) fn define_single_addr_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> SingleAddrVariable {
        self.define_variable(value_id, dfg).extract_single_addr()
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
                // Store non-param live-ins BEFORE converting the condition to avoid
                // register overwrites. Both branches' stores execute unconditionally.
                // Extra stores are harmless.
                self.spill_non_param_live_ins(*then_destination, dfg);
                self.spill_non_param_live_ins(*else_destination, dfg);
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
                // Store non-param live-ins BEFORE the arg/param parallel moves
                // to avoid register overwrites.
                self.spill_non_param_live_ins(*destination, dfg);
                let destination_block = &dfg[*destination];
                let mut moves: Vec<(MemoryAddress, MemoryAddress)> = Vec::new();
                for (arg, param) in arguments.iter().zip(destination_block.parameters()) {
                    let arg_var = self.convert_ssa_value(*arg, dfg);
                    let arg_reg = arg_var.extract_register();

                    // Check if the param was eagerly spilled as a successor block
                    // param. Use the permanent record (not the transient `spilled`
                    // map) so ALL Jmp sites consistently write to the spill slot,
                    // regardless of compilation order.
                    let spill_offset = self
                        .function_context
                        .spill_manager
                        .as_ref()
                        .and_then(|sm| sm.get_permanent_spill_offset(param));

                    if let Some(offset) = spill_offset {
                        // Param was spilled — write arg directly to param's spill slot.
                        self.function_context.did_spill = true;
                        self.function_context.max_spill_offset =
                            self.function_context.max_spill_offset.max(offset + 1);
                        let (scratch_addr, scratch_offset) = ReservedRegisters::spill_scratch();
                        self.brillig_context
                            .mov_instruction(scratch_addr, ReservedRegisters::spill_base_pointer());
                        self.brillig_context.const_instruction(
                            SingleAddrVariable::new_usize(scratch_offset),
                            offset.into(),
                        );
                        self.brillig_context.memory_op_instruction(
                            scratch_addr,
                            scratch_offset,
                            scratch_addr,
                            BrilligBinaryOp::Add,
                        );
                        self.brillig_context.store_instruction(scratch_addr, arg_reg);
                    } else {
                        let param_reg = self
                            .variables
                            .get_allocation(self.function_context, *param)
                            .extract_register();
                        moves.push((arg_reg, param_reg));
                    }
                }

                // Block parameter assignments at a jmp must happen "simultaneously".
                // A naive sequential loop can lose values when a source register
                // is overwritten by an earlier move in the same batch. For example, with:
                //   `jmp b1(v1, v2, u32 10)` where b1(v2, v3, v4):
                // Sequential execution would:
                //      1. mov reg(v2), reg(v1) — overwrites old v3
                //      2. mov reg(v3), reg(v2) — reads the NEW v2 instead of old
                // To prevent this, we save any source that would be overwritten into a
                // temporary first.
                let dest_set: HashSet<MemoryAddress> = moves.iter().map(|(_, d)| *d).collect();
                // `Allocated` automatically deallocates the register when dropped,
                // so we collect the temporaries here to keep them alive until all
                // moves have been emitted.
                let mut temps = Vec::new();
                for (src, _dst) in &mut moves {
                    if dest_set.contains(src) {
                        let temp = self.brillig_context.allocate_register();
                        self.brillig_context.mov_instruction(*temp, *src);
                        *src = *temp;
                        temps.push(temp);
                    }
                }

                for (src, dst) in &moves {
                    if src != dst {
                        self.brillig_context.mov_instruction(*dst, *src);
                    }
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
        let own_params: HashSet<ValueId> =
            dfg[self.block_id].parameters().iter().copied().collect();

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
                    self.define_variable(param_id, dfg);

                    // Successor block params (not this block's own params) don't hold
                    // valid data until a Jmp terminator writes to them. Eagerly spill
                    // them so that ALL Jmp sites consistently write to the spill slot
                    // and the target block consistently reloads from it.
                    if !own_params.contains(&param_id)
                        && let Some(sm) = self.function_context.spill_manager.as_mut()
                    {
                        sm.remove_from_lru(&param_id);
                        let offset = sm.allocate_spill_offset();
                        let var =
                            *self.function_context.ssa_value_allocations.get(&param_id).unwrap();
                        sm.record_permanent_spill(param_id, offset, var);
                        // Free the register — it holds no valid data.
                        let reg = var.extract_register();
                        self.brillig_context.deallocate_register(reg);
                        self.variables.mark_unavailable(&param_id);
                        self.function_context.did_spill = true;
                        self.function_context.max_spill_offset =
                            self.function_context.max_spill_offset.max(offset + 1);
                    }
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
        }

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
                    if self
                        .function_context
                        .spill_manager
                        .as_ref()
                        .is_some_and(|sm| sm.is_spilled(dead_variable))
                    {
                        // Spilled: register was already freed. Just clean up tracking.
                        let sm = self.function_context.spill_manager.as_mut().unwrap();
                        sm.remove_spill(dead_variable);
                        sm.remove_from_lru(dead_variable);
                        // Only remove from available_variables if it's actually there.
                        // A permanently spilled value may have been filtered out at block
                        // entry and never reloaded, so it was never in available_variables.
                        if self.variables.is_allocated(dead_variable) {
                            self.variables.mark_unavailable(dead_variable);
                        }
                    } else {
                        self.variables.remove_variable(
                            dead_variable,
                            self.function_context,
                            self.brillig_context,
                        );
                        if let Some(sm) = self.function_context.spill_manager.as_mut() {
                            sm.remove_from_lru(dead_variable);
                        }
                    }
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
                    // Check if spilled, reload if needed
                    if self
                        .function_context
                        .spill_manager
                        .as_ref()
                        .is_some_and(|sm| sm.is_spilled(&value_id))
                    {
                        return self.reload_spilled_value(value_id);
                    }
                    let var = self.variables.get_allocation(self.function_context, value_id);
                    if let Some(sm) = self.function_context.spill_manager.as_mut() {
                        sm.touch(value_id);
                    }
                    var
                }
            }
            Value::NumericConstant { constant, .. } => {
                // Check if spilled before is_allocated — spilled values are filtered
                // out of available_variables at block entry, so is_allocated would
                // return false even though the value exists.
                if self
                    .function_context
                    .spill_manager
                    .as_ref()
                    .is_some_and(|sm| sm.is_spilled(&value_id))
                {
                    return self.reload_spilled_value(value_id);
                }
                // Constants might have been converted previously or not, so we get or create and
                // (re)initialize the value inside.
                if self.variables.is_allocated(&value_id) {
                    let var = self.variables.get_allocation(self.function_context, value_id);
                    if let Some(sm) = self.function_context.spill_manager.as_mut() {
                        sm.touch(value_id);
                    }
                    var
                } else if dfg.is_global(value_id) {
                    *self.globals.get(&value_id).unwrap_or_else(|| {
                        panic!("ICE: Global value not found in cache {value_id}")
                    })
                } else {
                    let new_variable = self.define_variable(value_id, dfg);

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
                let new_variable = self.define_variable(value_id, dfg);

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
        if let Value::NumericConstant { constant, typ } = &dfg[value_id]
            && let Some(variable) = self.hoisted_global_constants.get(&(*constant, *typ))
        {
            return Some(*variable);
        }
        None
    }

    /// Define the result variable, convert the value, then generate Brillig opcodes to negate the variable.
    fn codegen_not(&mut self, instruction_id: InstructionId, value: ValueId, dfg: &DataFlowGraph) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let condition_register = self.convert_ssa_single_addr_value(value, dfg);
        let result_register = self.define_single_addr_variable(result_id, dfg);
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
        let destination_register = self.define_single_addr_variable(result_id, dfg);
        let source_register = self.convert_ssa_single_addr_value(value, dfg);
        self.brillig_context.codegen_truncate(destination_register, source_register, bit_size);
    }

    /// Define the result variable, convert the value, then generate Brillig opcodes to truncate the variable.
    fn codegen_cast(&mut self, instruction_id: InstructionId, value: ValueId, dfg: &DataFlowGraph) {
        let [result_id] = dfg.instruction_result(instruction_id);
        let destination_variable = self.define_single_addr_variable(result_id, dfg);
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
        let result = self.define_variable(result_id, dfg);

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

/// Filter a block's live-in set to exclude globals and hoisted global constants.
fn live_in_no_globals(
    live_in: &HashSet<ValueId>,
    dfg: &DataFlowGraph,
    hoisted_global_constants: &HoistedConstantsToBrilligGlobals,
) -> HashSet<ValueId> {
    live_in
        .iter()
        .copied()
        .filter(|&value| {
            if let Value::NumericConstant { constant, typ } = dfg[value]
                && hoisted_global_constants.contains_key(&(constant, typ))
            {
                return false;
            }
            !dfg.is_global(value)
        })
        .collect()
}
