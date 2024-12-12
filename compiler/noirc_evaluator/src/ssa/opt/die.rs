//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::{CallStack, DataFlowGraph},
        function::Function,
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic},
        post_order::PostOrder,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

use super::rc::{pop_rc_for, RcInstruction};

impl Ssa {
    /// Performs Dead Instruction Elimination (DIE) to remove any instructions with
    /// unused results.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn dead_instruction_elimination(mut self) -> Ssa {
        self.functions.par_iter_mut().for_each(|(_, func)| func.dead_instruction_elimination(true));

        self
    }
}

impl Function {
    /// Removes any unused instructions in the reachable blocks of the given function.
    ///
    /// The blocks of the function are iterated in post order, such that any blocks containing
    /// instructions that reference results from an instruction in another block are evaluated first.
    /// If we did not iterate blocks in this order we could not safely say whether or not the results
    /// of its instructions are needed elsewhere.
    pub(crate) fn dead_instruction_elimination(&mut self, insert_out_of_bounds_checks: bool) {
        let mut context = Context::default();
        for call_data in &self.dfg.data_bus.call_data {
            context.mark_used_instruction_results(&self.dfg, call_data.array_id);
        }

        let mut inserted_out_of_bounds_checks = false;

        let blocks = PostOrder::with_function(self);
        for block in blocks.as_slice() {
            inserted_out_of_bounds_checks |= context.remove_unused_instructions_in_block(
                self,
                *block,
                insert_out_of_bounds_checks,
            );
        }

        // If we inserted out of bounds check, let's run the pass again with those new
        // instructions (we don't want to remove those checks, or instructions that are
        // dependencies of those checks)
        if inserted_out_of_bounds_checks {
            self.dead_instruction_elimination(false);
            return;
        }

        context.remove_rc_instructions(&mut self.dfg);
    }
}

/// Per function context for tracking unused values and which instructions to remove.
#[derive(Default)]
struct Context {
    used_values: HashSet<ValueId>,
    instructions_to_remove: HashSet<InstructionId>,

    /// IncrementRc & DecrementRc instructions must be revisited after the main DIE pass since
    /// they technically contain side-effects but we still want to remove them if their
    /// `value` parameter is not used elsewhere.
    rc_instructions: Vec<(InstructionId, BasicBlockId)>,
}

impl Context {
    /// Steps backwards through the instruction of the given block, amassing a set of used values
    /// as it goes, and at the same time marking instructions for removal if they haven't appeared
    /// in the set thus far.
    ///
    /// It is not only safe to mark instructions for removal as we go because no instruction
    /// result value can be referenced before the occurrence of the instruction that produced it,
    /// and we are iterating backwards. It is also important to identify instructions that can be
    /// removed as we go, such that we know not to include its referenced values in the used
    /// values set. This allows DIE to identify whole chains of unused instructions. (If the
    /// values referenced by an unused instruction were considered to be used, only the head of
    /// such chains would be removed.)
    ///
    /// If `insert_out_of_bounds_checks` is true and there are unused ArrayGet/ArraySet that
    /// might be out of bounds, this method will insert out of bounds checks instead of
    /// removing unused instructions and return `true`. The idea then is to later call this
    /// function again with `insert_out_of_bounds_checks` set to false to effectively remove
    /// unused instructions but leave the out of bounds checks.
    fn remove_unused_instructions_in_block(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        insert_out_of_bounds_checks: bool,
    ) -> bool {
        let block = &function.dfg[block_id];
        self.mark_terminator_values_as_used(function, block);

        let instructions_len = block.instructions().len();

        let mut rc_tracker = RcTracker::default();

        // Indexes of instructions that might be out of bounds.
        // We'll remove those, but before that we'll insert bounds checks for them.
        let mut possible_index_out_of_bounds_indexes = Vec::new();

        for (instruction_index, instruction_id) in block.instructions().iter().rev().enumerate() {
            let instruction = &function.dfg[*instruction_id];

            if self.is_unused(*instruction_id, function) {
                self.instructions_to_remove.insert(*instruction_id);

                if insert_out_of_bounds_checks
                    && instruction_might_result_in_out_of_bounds(function, instruction)
                {
                    possible_index_out_of_bounds_indexes
                        .push(instructions_len - instruction_index - 1);
                }
            } else {
                // We can't remove rc instructions if they're loaded from a reference
                // since we'd have no way of knowing whether the reference is still used.
                if Self::is_inc_dec_instruction_on_known_array(instruction, &function.dfg) {
                    self.rc_instructions.push((*instruction_id, block_id));
                } else {
                    instruction.for_each_value(|value| {
                        self.mark_used_instruction_results(&function.dfg, value);
                    });
                }
            }

            rc_tracker.track_inc_rcs_to_remove(*instruction_id, function);
        }

        self.instructions_to_remove.extend(rc_tracker.get_non_mutated_arrays(&function.dfg));
        self.instructions_to_remove.extend(rc_tracker.rc_pairs_to_remove);

        // If there are some instructions that might trigger an out of bounds error,
        // first add constrain checks. Then run the DIE pass again, which will remove those
        // but leave the constrains (any any value needed by those constrains)
        if !possible_index_out_of_bounds_indexes.is_empty() {
            let inserted_check = self.replace_array_instructions_with_out_of_bounds_checks(
                function,
                block_id,
                &mut possible_index_out_of_bounds_indexes,
            );
            // There's a slight chance we didn't insert any checks, so we could proceed with DIE.
            if inserted_check {
                return true;
            }
        }

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));

        false
    }

    /// Returns true if an instruction can be removed.
    ///
    /// An instruction can be removed as long as it has no side-effects, and none of its result
    /// values have been referenced.
    fn is_unused(&self, instruction_id: InstructionId, function: &Function) -> bool {
        let instruction = &function.dfg[instruction_id];

        if instruction.can_eliminate_if_unused(function) {
            let results = function.dfg.instruction_results(instruction_id);
            results.iter().all(|result| !self.used_values.contains(result))
        } else if let Instruction::Call { func, arguments } = instruction {
            // TODO: make this more general for instructions which don't have results but have side effects "sometimes" like `Intrinsic::AsWitness`
            let as_witness_id = function.dfg.get_intrinsic(Intrinsic::AsWitness);
            as_witness_id == Some(func) && !self.used_values.contains(&arguments[0])
        } else {
            // If the instruction has side effects we should never remove it.
            false
        }
    }

    /// Adds values referenced by the terminator to the set of used values.
    fn mark_terminator_values_as_used(&mut self, function: &Function, block: &BasicBlock) {
        block.unwrap_terminator().for_each_value(|value| {
            self.mark_used_instruction_results(&function.dfg, value);
        });
    }

    /// Inspects a value and marks all instruction results as used.
    fn mark_used_instruction_results(&mut self, dfg: &DataFlowGraph, value_id: ValueId) {
        let value_id = dfg.resolve(value_id);
        if matches!(&dfg[value_id], Value::Instruction { .. } | Value::Param { .. }) {
            self.used_values.insert(value_id);
        }
    }

    fn remove_rc_instructions(self, dfg: &mut DataFlowGraph) {
        let unused_rc_values_by_block: HashMap<BasicBlockId, HashSet<InstructionId>> =
            self.rc_instructions.into_iter().fold(HashMap::default(), |mut acc, (rc, block)| {
                let value = match &dfg[rc] {
                    Instruction::IncrementRc { value } => *value,
                    Instruction::DecrementRc { value } => *value,
                    other => {
                        unreachable!(
                            "Expected IncrementRc or DecrementRc instruction, found {other:?}"
                        )
                    }
                };

                if !self.used_values.contains(&value) {
                    acc.entry(block).or_default().insert(rc);
                }
                acc
            });

        for (block, instructions_to_remove) in unused_rc_values_by_block {
            dfg[block]
                .instructions_mut()
                .retain(|instruction| !instructions_to_remove.contains(instruction));
        }
    }

    /// Replaces unused ArrayGet/ArraySet instructions with out of bounds checks.
    /// Returns `true` if at least one check was inserted.
    /// Because some ArrayGet might happen in groups (for composite types), if just
    /// some of the instructions in a group are used but not all of them, no check
    /// is inserted, so this method might return `false`.
    fn replace_array_instructions_with_out_of_bounds_checks(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        possible_index_out_of_bounds_indexes: &mut Vec<usize>,
    ) -> bool {
        let mut inserted_check = false;

        // Keep track of the current side effects condition
        let mut side_effects_condition = None;

        // Keep track of the next index we need to handle
        let mut next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();

        let instructions = function.dfg[block_id].take_instructions();
        for (index, instruction_id) in instructions.iter().enumerate() {
            let instruction_id = *instruction_id;
            let instruction = &function.dfg[instruction_id];

            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                side_effects_condition = Some(*condition);

                // We still need to keep the EnableSideEffects instruction
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            };

            // If it's an ArrayGet we'll deal with groups of it in case the array type is a composite type,
            // and adjust `next_out_of_bounds_index` and `possible_index_out_of_bounds_indexes` accordingly
            if let Instruction::ArrayGet { array, .. } = instruction {
                handle_array_get_group(
                    function,
                    array,
                    index,
                    &mut next_out_of_bounds_index,
                    possible_index_out_of_bounds_indexes,
                );
            }

            let Some(out_of_bounds_index) = next_out_of_bounds_index else {
                // No more out of bounds instructions to insert, just push the current instruction
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            };

            if index != out_of_bounds_index {
                // This instruction is not out of bounds: let's just push it
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            }

            // This is an instruction that might be out of bounds: let's add a constrain.
            let (array, index) = match instruction {
                Instruction::ArrayGet { array, index }
                | Instruction::ArraySet { array, index, .. } => (array, index),
                _ => panic!("Expected an ArrayGet or ArraySet instruction here"),
            };

            let call_stack = function.dfg.get_call_stack(instruction_id);

            let (lhs, rhs) = if function.dfg.get_numeric_constant(*index).is_some() {
                // If we are here it means the index is known but out of bounds. That's always an error!
                let false_const = function.dfg.make_constant(false.into(), NumericType::bool());
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (false_const, true_const)
            } else {
                // `index` will be relative to the flattened array length, so we need to take that into account
                let array_length = function.dfg.type_of_value(*array).flattened_size();

                // If we are here it means the index is dynamic, so let's add a check that it's less than length
                let length_type = NumericType::length_type();
                let index = function.dfg.insert_instruction_and_results(
                    Instruction::Cast(*index, length_type),
                    block_id,
                    None,
                    call_stack.clone(),
                );
                let index = index.first();

                let array_length =
                    function.dfg.make_constant((array_length as u128).into(), length_type);
                let is_index_out_of_bounds = function.dfg.insert_instruction_and_results(
                    Instruction::binary(BinaryOp::Lt, index, array_length),
                    block_id,
                    None,
                    call_stack.clone(),
                );
                let is_index_out_of_bounds = is_index_out_of_bounds.first();
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (is_index_out_of_bounds, true_const)
            };

            let (lhs, rhs) = apply_side_effects(
                side_effects_condition,
                lhs,
                rhs,
                function,
                block_id,
                call_stack.clone(),
            );

            let message = Some("Index out of bounds".to_owned().into());
            function.dfg.insert_instruction_and_results(
                Instruction::Constrain(lhs, rhs, message),
                block_id,
                None,
                call_stack,
            );
            inserted_check = true;

            next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
        }

        inserted_check
    }

    /// True if this is a `Instruction::IncrementRc` or `Instruction::DecrementRc`
    /// operating on an array directly from a `Instruction::MakeArray` or an
    /// intrinsic known to return a fresh array.
    fn is_inc_dec_instruction_on_known_array(
        instruction: &Instruction,
        dfg: &DataFlowGraph,
    ) -> bool {
        use Instruction::*;
        if let IncrementRc { value } | DecrementRc { value } = instruction {
            if let Value::Instruction { instruction, .. } = &dfg[*value] {
                return match &dfg[*instruction] {
                    MakeArray { .. } => true,
                    Call { func, .. } => {
                        matches!(&dfg[*func], Value::Intrinsic(_) | Value::ForeignFunction(_))
                    }
                    _ => false,
                };
            }
        }
        false
    }
}

fn instruction_might_result_in_out_of_bounds(
    function: &Function,
    instruction: &Instruction,
) -> bool {
    use Instruction::*;
    match instruction {
        ArrayGet { array, index } | ArraySet { array, index, .. } => {
            if function.dfg.try_get_array_length(*array).is_some() {
                if let Some(known_index) = function.dfg.get_numeric_constant(*index) {
                    // `index` will be relative to the flattened array length, so we need to take that into account
                    let typ = function.dfg.type_of_value(*array);
                    let array_length = typ.flattened_size();
                    known_index >= array_length.into()
                } else {
                    // A dynamic index might always be out of bounds
                    true
                }
            } else {
                // Slice operations might be out of bounds, but there's no way we
                // can insert a check because we don't know a slice's length
                false
            }
        }
        _ => false,
    }
}

fn handle_array_get_group(
    function: &Function,
    array: &ValueId,
    index: usize,
    next_out_of_bounds_index: &mut Option<usize>,
    possible_index_out_of_bounds_indexes: &mut Vec<usize>,
) {
    if function.dfg.try_get_array_length(*array).is_none() {
        // Nothing to do for slices
        return;
    };

    let element_size = function.dfg.type_of_value(*array).element_size();
    if element_size <= 1 {
        // Not a composite type
        return;
    };

    // It's a composite type.
    // When doing ArrayGet on a composite type, this **always** results in instructions like these
    // (assuming element_size == 3):
    //
    // 1.    v27 = array_get v1, index v26
    // 2.    v28 = add v26, u32 1
    // 3.    v29 = array_get v1, index v28
    // 4.    v30 = add v26, u32 2
    // 5.    v31 = array_get v1, index v30
    //
    // That means that after this instructions, (element_size - 1) instructions will be
    // part of this composite array get, and they'll be two instructions apart.
    //
    // Now three things can happen:
    // a) none of the array_get instructions are unused: in this case they won't be in
    //    `possible_index_out_of_bounds_indexes` and they won't be removed, nothing to do here
    // b) all of the array_get instructions are unused: in this case we can replace **all**
    //    of them with just one constrain: no need to do one per array_get
    // c) some of the array_get instructions are unused, but not all: in this case
    //    we don't need to insert any constrain, because on a later stage array bound checks
    //    will be performed anyway. We'll let DIE remove the unused ones, without replacing
    //    them with bounds checks, and leave the used ones.
    //
    // To check in which scenario we are we can get from `possible_index_out_of_bounds_indexes`
    // (starting from `next_out_of_bounds_index`) while we are in the group ranges
    // (1..=5 in the example above)

    let Some(out_of_bounds_index) = *next_out_of_bounds_index else {
        // No next unused instruction, so this is case a) and nothing needs to be done here
        return;
    };

    if index != out_of_bounds_index {
        // The next index is not the one for the current instructions,
        // so we are in case a), and nothing needs to be done here
        return;
    }

    // What's the last instruction that's part of the group? (5 in the example above)
    let last_instruction_index = index + 2 * (element_size - 1);
    // How many unused instructions are in this group?
    let mut unused_count = 1;
    loop {
        *next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
        if let Some(out_of_bounds_index) = *next_out_of_bounds_index {
            if out_of_bounds_index <= last_instruction_index {
                unused_count += 1;
                if unused_count == element_size {
                    // We are in case b): we need to insert just one constrain.
                    // Since we popped all of the group indexes, and given that we
                    // are analyzing the first instruction in the group, we can
                    // set `next_out_of_bounds_index` to the current index:
                    // then a check will be inserted, and no other check will be
                    // inserted for the rest of the group.
                    *next_out_of_bounds_index = Some(index);
                    break;
                } else {
                    continue;
                }
            }
        }

        // We are in case c): some of the instructions are unused.
        // We don't need to insert any checks, and given that we already popped
        // all of the indexes in the group, there's nothing else to do here.
        break;
    }
}

// Given `lhs` and `rhs` values, if there's a side effects condition this will
// return (`lhs * condition`, `rhs * condition`), otherwise just (`lhs`, `rhs`)
fn apply_side_effects(
    side_effects_condition: Option<ValueId>,
    lhs: ValueId,
    rhs: ValueId,
    function: &mut Function,
    block_id: BasicBlockId,
    call_stack: CallStack,
) -> (ValueId, ValueId) {
    // See if there's an active "enable side effects" condition
    let Some(condition) = side_effects_condition else {
        return (lhs, rhs);
    };

    let dfg = &mut function.dfg;

    // Condition needs to be cast to argument type in order to multiply them together.
    // In our case, lhs is always a boolean.
    let cast = Instruction::Cast(condition, NumericType::bool());
    let casted_condition =
        dfg.insert_instruction_and_results(cast, block_id, None, call_stack.clone());
    let casted_condition = casted_condition.first();

    let lhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul, lhs, casted_condition),
        block_id,
        None,
        call_stack.clone(),
    );
    let lhs = lhs.first();

    let rhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul, rhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let rhs = rhs.first();

    (lhs, rhs)
}

#[derive(Default)]
struct RcTracker {
    // We can track IncrementRc instructions per block to determine whether they are useless.
    // IncrementRc and DecrementRc instructions are normally side effectual instructions, but we remove
    // them if their value is not used anywhere in the function. However, even when their value is used, their existence
    // is pointless logic if there is no array set between the increment and the decrement of the reference counter.
    // We track per block whether an IncrementRc instruction has a paired DecrementRc instruction
    // with the same value but no array set in between.
    // If we see an inc/dec RC pair within a block we can safely remove both instructions.
    rcs_with_possible_pairs: HashMap<Type, Vec<RcInstruction>>,
    rc_pairs_to_remove: HashSet<InstructionId>,
    // We also separately track all IncrementRc instructions and all arrays which have been mutably borrowed.
    // If an array has not been mutably borrowed we can then safely remove all IncrementRc instructions on that array.
    inc_rcs: HashMap<ValueId, HashSet<InstructionId>>,
    mutated_array_types: HashSet<Type>,
    // The SSA often creates patterns where after simplifications we end up with repeat
    // IncrementRc instructions on the same value. We track whether the previous instruction was an IncrementRc,
    // and if the current instruction is also an IncrementRc on the same value we remove the current instruction.
    // `None` if the previous instruction was anything other than an IncrementRc
    previous_inc_rc: Option<ValueId>,
}

impl RcTracker {
    fn track_inc_rcs_to_remove(&mut self, instruction_id: InstructionId, function: &Function) {
        let instruction = &function.dfg[instruction_id];

        if let Instruction::IncrementRc { value } = instruction {
            if let Some(previous_value) = self.previous_inc_rc {
                if previous_value == *value {
                    self.rc_pairs_to_remove.insert(instruction_id);
                }
            }
            self.previous_inc_rc = Some(*value);
        } else {
            self.previous_inc_rc = None;
        }

        // DIE loops over a block in reverse order, so we insert an RC instruction for possible removal
        // when we see a DecrementRc and check whether it was possibly mutated when we see an IncrementRc.
        match instruction {
            Instruction::IncrementRc { value } => {
                if let Some(inc_rc) =
                    pop_rc_for(*value, function, &mut self.rcs_with_possible_pairs)
                {
                    if !inc_rc.possibly_mutated {
                        self.rc_pairs_to_remove.insert(inc_rc.id);
                        self.rc_pairs_to_remove.insert(instruction_id);
                    }
                }

                self.inc_rcs.entry(*value).or_default().insert(instruction_id);
            }
            Instruction::DecrementRc { value } => {
                let typ = function.dfg.type_of_value(*value);

                // We assume arrays aren't mutated until we find an array_set
                let dec_rc =
                    RcInstruction { id: instruction_id, array: *value, possibly_mutated: false };
                self.rcs_with_possible_pairs.entry(typ).or_default().push(dec_rc);
            }
            Instruction::ArraySet { array, .. } => {
                let typ = function.dfg.type_of_value(*array);
                if let Some(dec_rcs) = self.rcs_with_possible_pairs.get_mut(&typ) {
                    for dec_rc in dec_rcs {
                        dec_rc.possibly_mutated = true;
                    }
                }

                self.mutated_array_types.insert(typ);
            }
            Instruction::Store { value, .. } => {
                // We are very conservative and say that any store of an array type means it has the potential to be mutated.
                let typ = function.dfg.type_of_value(*value);
                if matches!(&typ, Type::Array(..) | Type::Slice(..)) {
                    self.mutated_array_types.insert(typ);
                }
            }
            Instruction::Call { arguments, .. } => {
                for arg in arguments {
                    let typ = function.dfg.type_of_value(*arg);
                    if matches!(&typ, Type::Array(..) | Type::Slice(..)) {
                        self.mutated_array_types.insert(typ);
                    }
                }
            }
            _ => {}
        }
    }

    fn get_non_mutated_arrays(&self, dfg: &DataFlowGraph) -> HashSet<InstructionId> {
        self.inc_rcs
            .keys()
            .filter_map(|value| {
                let typ = dfg.type_of_value(*value);
                if !self.mutated_array_types.contains(&typ) {
                    Some(&self.inc_rcs[value])
                } else {
                    None
                }
            })
            .flatten()
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use im::vector;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            map::Id,
            types::{NumericType, Type},
        },
        opt::assert_normalized_ssa_equals,
        Ssa,
    };

    #[test]
    fn dead_instruction_elimination() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v3 = add v0, Field 1
                v5 = add v0, Field 2
                jmp b1(v5)
              b1(v1: Field):
                v6 = allocate -> &mut Field
                v7 = load v6 -> Field
                v8 = allocate -> &mut Field
                store Field 1 at v8
                v9 = load v8 -> Field
                v10 = add v9, Field 1
                v11 = add v9, Field 2
                v13 = add v9, Field 3
                v14 = add v13, v13
                call assert_constant(v10)
                return v11
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v3 = add v0, Field 2
                jmp b1(v3)
              b1(v1: Field):
                v4 = allocate -> &mut Field
                store Field 1 at v4
                v6 = load v4 -> Field
                v7 = add v6, Field 1
                v8 = add v6, Field 2
                call assert_constant(v7)
                return v8
            }
            ";
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn as_witness_die() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v2 = add v0, Field 1
                v4 = add v0, Field 2
                call as_witness(v4)
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v2 = add v0, Field 1
                return v2
            }
            ";
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn keep_paired_rcs_with_array_set() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v2 = array_set v0, index u32 0, value u32 0
                dec_rc v0
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        // We expect the output to be unchanged
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn keep_inc_rc_on_borrowed_array_store() {
        // acir(inline) fn main f0 {
        //     b0():
        //       v1 = make_array [u32 0, u32 0]
        //       v2 = allocate
        //       inc_rc v1
        //       store v1 at v2
        //       inc_rc v1
        //       jmp b1()
        //     b1():
        //       v3 = load v2
        //       v5 = array_set v3, index u32 0, value u32 1
        //       return v5
        //   }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let zero = builder.numeric_constant(0u128, NumericType::unsigned(32));
        let array_type = Type::Array(Arc::new(vec![Type::unsigned(32)]), 2);
        let v1 = builder.insert_make_array(vector![zero, zero], array_type.clone());
        let v2 = builder.insert_allocate(array_type.clone());
        builder.increment_array_reference_count(v1);
        builder.insert_store(v2, v1);
        builder.increment_array_reference_count(v1);

        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![]);
        builder.switch_to_block(b1);

        let v3 = builder.insert_load(v2, array_type);
        let one = builder.numeric_constant(1u128, NumericType::unsigned(32));
        let v5 = builder.insert_array_set(v3, zero, one);
        builder.terminate_with_return(vec![v5]);

        let ssa = builder.finish();
        let main = ssa.main();

        // The instruction count never includes the terminator instruction
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 5);
        assert_eq!(main.dfg[b1].instructions().len(), 2);

        // We expect the output to be unchanged
        let ssa = ssa.dead_instruction_elimination();
        let main = ssa.main();

        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 5);
        assert_eq!(main.dfg[b1].instructions().len(), 2);
    }

    #[test]
    fn does_not_remove_inc_or_dec_rc_of_if_they_are_loaded_from_a_reference() {
        let src = "
            brillig(inline) fn borrow_mut f0 {
              b0(v0: &mut [Field; 3]):
                v1 = load v0 -> [Field; 3]
                inc_rc v1 // this one shouldn't be removed
                v2 = load v0 -> [Field; 3]
                inc_rc v2 // this one shouldn't be removed
                v3 = load v0 -> [Field; 3]
                v6 = array_set v3, index u32 0, value Field 5
                store v6 at v0
                dec_rc v6
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, src);
    }
}
