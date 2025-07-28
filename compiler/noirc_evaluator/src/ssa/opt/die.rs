//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
//!
//! DIE also tracks which block parameters are unused.
//! Unused parameters are then pruned by the [prune_dead_parameters] pass.
use acvm::{AcirField, FieldElement, acir::BlackBoxFunc};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use noirc_errors::call_stack::CallStackId;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        post_order::PostOrder,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    opt::pure::Purity,
    ssa_gen::Ssa,
};

use super::rc::{RcInstruction, pop_rc_for};

mod prune_dead_parameters;

impl Ssa {
    /// Performs Dead Instruction Elimination (DIE) to remove any instructions with
    /// unused results.
    ///
    /// This step should come after the flattening of the CFG and mem2reg.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn dead_instruction_elimination(self) -> Ssa {
        self.dead_instruction_elimination_with_pruning(true, false)
    }

    /// Post the Brillig generation we do not need to run this pass on Brillig functions.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn dead_instruction_elimination_acir(self) -> Ssa {
        self.dead_instruction_elimination_with_pruning(true, true)
    }

    /// The elimination of certain unused instructions assumes that the DIE pass runs after
    /// the flattening of the CFG, but if that's not the case then we should not eliminate
    /// them just yet.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn dead_instruction_elimination_pre_flattening(self) -> Ssa {
        self.dead_instruction_elimination_with_pruning(false, false)
    }

    fn dead_instruction_elimination_with_pruning(
        mut self,
        flattened: bool,
        skip_brillig: bool,
    ) -> Ssa {
        let mut previous_unused_params = None;
        loop {
            let (new_ssa, result) =
                self.dead_instruction_elimination_inner(flattened, skip_brillig);

            // Determine whether we have any unused variables
            let has_unused = result
                .unused_parameters
                .values()
                .any(|block_map| block_map.values().any(|params| !params.is_empty()));

            // If there are no unused parameters, return early
            if !has_unused {
                return new_ssa;
            }

            if let Some(previous) = &previous_unused_params {
                // If no changes to dead parameters occurred, return early
                if previous == &result.unused_parameters {
                    return new_ssa;
                }
            }

            // Prune unused parameters and repeat
            self = new_ssa.prune_dead_parameters(&result.unused_parameters);
            previous_unused_params = Some(result.unused_parameters);
        }
    }

    fn dead_instruction_elimination_inner(
        mut self,
        flattened: bool,
        skip_brillig: bool,
    ) -> (Ssa, DIEResult) {
        let result = self
            .functions
            .par_iter_mut()
            .map(|(id, func)| {
                let unused_params =
                    func.dead_instruction_elimination(true, flattened, skip_brillig);
                let mut result = DIEResult::default();

                result.unused_parameters.insert(*id, unused_params);

                result
            })
            .reduce(DIEResult::default, |mut a, b| {
                a.unused_parameters.extend(b.unused_parameters);
                a
            });

        (self, result)
    }

    /// Sanity check on the final SSA, panicking if the assumptions don't hold.
    ///
    /// Done as a separate step so that we can put it after other passes which provide
    /// concrete feedback about where the problem with the Noir code might be, such as
    /// dynamic indexing of arrays with references in ACIR. We can look up the callstack
    /// of the offending instruction here as well, it's just not clear what error message
    /// to return, besides the fact that mem2reg was unable to eliminate something.
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    pub(crate) fn dead_instruction_elimination_post_check(&self, flattened: bool) {
        #[cfg(debug_assertions)]
        self.functions.values().for_each(|f| die_post_check(f, flattened));
    }
}

impl Function {
    /// Removes any unused instructions in the reachable blocks of the given function.
    ///
    /// This method is designed to be run within the context of the full SSA, not in isolation.
    /// Running DIE on a single function may cause inconsistencies, such as leaving dangling unused parameters.
    /// The pruning of block parameters depends on the full SSA context.
    /// Therefore, this method must remain private, and DIE should run over the entire SSA,
    /// ensuring proper tracking of unused parameters across all blocks.
    ///
    /// The blocks of the function are iterated in post order, such that any blocks containing
    /// instructions that reference results from an instruction in another block are evaluated first.
    /// If we did not iterate blocks in this order we could not safely say whether or not the results
    /// of its instructions are needed elsewhere.
    ///
    /// # Returns
    ///   After processing all functions, the union of these sets enables determining the unused globals.
    /// - A mapping of (block id -> unused parameters) for the given function.
    ///   This can be used by follow-up passes to prune unused parameters from blocks.
    fn dead_instruction_elimination(
        &mut self,
        insert_out_of_bounds_checks: bool,
        flattened: bool,
        skip_brillig: bool,
    ) -> HashMap<BasicBlockId, Vec<ValueId>> {
        if skip_brillig && self.dfg.runtime().is_brillig() {
            return HashMap::default();
        }

        let mut context = Context { flattened, ..Default::default() };

        context.mark_function_parameter_arrays_as_used(self);

        for call_data in &self.dfg.data_bus.call_data {
            context.mark_used_instruction_results(&self.dfg, call_data.array_id);
        }

        let mut inserted_out_of_bounds_checks = false;

        let blocks = PostOrder::with_function(self);
        let mut unused_params_per_block = HashMap::default();
        for block in blocks.as_slice() {
            inserted_out_of_bounds_checks |= context.remove_unused_instructions_in_block(
                self,
                *block,
                insert_out_of_bounds_checks,
            );

            let parameters = self.dfg[*block].parameters();
            let mut keep_list = Vec::with_capacity(parameters.len());
            let unused_params = parameters
                .iter()
                .filter(|value| {
                    let keep = context.used_values.contains(value);
                    keep_list.push(keep);
                    !keep
                })
                .copied()
                .collect::<Vec<_>>();

            unused_params_per_block.insert(*block, unused_params);
            context.parameter_keep_list.insert(*block, keep_list);
        }

        // If we inserted out of bounds check, let's run the pass again with those new
        // instructions (we don't want to remove those checks, or instructions that are
        // dependencies of those checks)
        if inserted_out_of_bounds_checks {
            return self.dead_instruction_elimination(false, flattened, skip_brillig);
        }

        context.remove_rc_instructions(&mut self.dfg);

        unused_params_per_block
    }
}

#[derive(Default)]
struct DIEResult {
    unused_parameters: HashMap<FunctionId, HashMap<BasicBlockId, Vec<ValueId>>>,
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

    /// The elimination of certain unused instructions assumes that the DIE pass runs after
    /// the flattening of the CFG, but if that's not the case then we should not eliminate
    /// them just yet.
    flattened: bool,

    /// Track IncrementRc instructions per block to determine whether they are useless.
    rc_tracker: RcTracker,

    /// A per-block list indicating which block parameters are still considered alive.
    ///
    /// Each entry maps a [BasicBlockId] to a `Vec<bool>`, where the `i`th boolean corresponds to
    /// the `i`th parameter of that block. A value of `true` means the parameter is used and should
    /// be preserved. A value of `false` means it is unused and can be pruned.
    ///
    /// This keep list is used during terminator analysis to avoid incorrectly marking values as used
    /// simply because they appear as terminator arguments. Only parameters marked as live here
    /// should result in values being marked as used in terminator arguments.
    parameter_keep_list: HashMap<BasicBlockId, Vec<bool>>,
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

        self.rc_tracker.new_block();
        self.rc_tracker.mark_terminator_arrays_as_used(function, block);

        let instructions_len = block.instructions().len();

        // Indexes of instructions that might be out of bounds.
        // We'll remove those, but before that we'll insert bounds checks for them.
        let mut possible_index_out_of_bounds_indexes = Vec::new();

        // Going in reverse so we know if a result of an instruction was used.
        for (instruction_index, instruction_id) in block.instructions().iter().rev().enumerate() {
            let instruction = &function.dfg[*instruction_id];

            if self.is_unused(*instruction_id, function) {
                self.instructions_to_remove.insert(*instruction_id);

                // Array get/set has explicit out of bounds (OOB) checks laid down in the Brillig runtime.
                // These checks are not laid down in the ACIR runtime as that runtime maps the SSA
                // to a memory model where OOB accesses will be prevented. Essentially all array ops
                // in ACIR will have a side effect where they check for the index being OOB.
                // However, in order to maintain parity between the Brillig and ACIR runtimes,
                // if we have an unused array operation we need insert an OOB check so that the
                // side effects ordering remains correct.
                if function.runtime().is_acir()
                    && insert_out_of_bounds_checks
                    && instruction_might_result_in_out_of_bounds(function, instruction)
                {
                    possible_index_out_of_bounds_indexes
                        .push(instructions_len - instruction_index - 1);
                    // We need to still mark the array index as used as we refer to it in the inserted bounds check.
                    let (Instruction::ArrayGet { index, .. } | Instruction::ArraySet { index, .. }) =
                        instruction
                    else {
                        unreachable!("Only enter this branch on array gets/sets")
                    };
                    self.mark_used_instruction_results(&function.dfg, *index);
                }
            } else {
                // We can't remove rc instructions if they're loaded from a reference
                // since we'd have no way of knowing whether the reference is still used.
                if Self::is_inc_dec_instruction_on_known_array(instruction, &function.dfg) {
                    dbg!(instruction.clone());
                    self.rc_instructions.push((*instruction_id, block_id));
                } else {
                    instruction.for_each_value(|value| {
                        self.mark_used_instruction_results(&function.dfg, value);
                    });
                }
            }

            self.rc_tracker.track_inc_rcs_to_remove(*instruction_id, function);
        }

        self.instructions_to_remove.extend(self.rc_tracker.get_non_mutated_arrays(&function.dfg));
        self.instructions_to_remove.extend(self.rc_tracker.rc_pairs_to_remove.drain());

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

        if can_be_eliminated_if_unused(instruction, function, self.flattened) {
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
        let terminator = block.unwrap_terminator();
        let jmp_destination = if let TerminatorInstruction::Jmp { destination, .. } = terminator {
            Some(*destination)
        } else {
            None
        };

        block.unwrap_terminator().for_eachi_value(|index, value| {
            let keep_list = jmp_destination.and_then(|dest| self.parameter_keep_list.get(&dest));
            let should_keep = keep_list.is_none_or(|list| list[index]);
            if should_keep {
                self.mark_used_instruction_results(&function.dfg, value);
            }
        });
    }

    /// Inspects a value and marks all instruction results as used.
    fn mark_used_instruction_results(&mut self, dfg: &DataFlowGraph, value_id: ValueId) {
        if matches!(&dfg[value_id], Value::Instruction { .. } | Value::Param { .. })
            || dfg.is_global(value_id)
        {
            self.used_values.insert(value_id);
        }
    }

    /// Mark any array parameters to the function itself as possibly mutated.
    fn mark_function_parameter_arrays_as_used(&mut self, function: &Function) {
        for parameter in function.parameters() {
            let typ = function.dfg.type_of_value(*parameter);
            if typ.contains_an_array() {
                let typ = typ.get_contained_array();
                // Want to store the array type which is being referenced,
                // because it's the underlying array that the `inc_rc` is associated with.
                self.add_mutated_array_type(typ.clone());
            }
        }
    }

    fn add_mutated_array_type(&mut self, typ: Type) {
        self.rc_tracker.mutated_array_types.insert(typ.get_contained_array().clone());
    }

    /// Go through the RC instructions collected when we figured out which values were unused;
    /// for each RC that refers to an unused value, remove the RC as well.
    fn remove_rc_instructions(&self, dfg: &mut DataFlowGraph) {
        let unused_rc_values_by_block: HashMap<BasicBlockId, HashSet<InstructionId>> =
            self.rc_instructions.iter().fold(HashMap::default(), |mut acc, (rc, block)| {
                let value = match &dfg[*rc] {
                    Instruction::IncrementRc { value } => *value,
                    Instruction::DecrementRc { value, .. } => *value,
                    other => {
                        unreachable!(
                            "Expected IncrementRc or DecrementRc instruction, found {other:?}"
                        )
                    }
                };

                if !self.used_values.contains(&value) {
                    acc.entry(*block).or_default().insert(*rc);
                }
                acc
            });

        for (block, instructions_to_remove) in unused_rc_values_by_block {
            dfg[block]
                .instructions_mut()
                .retain(|instruction| !instructions_to_remove.contains(instruction));
        }
    }

    /// True if this is a `Instruction::IncrementRc` or `Instruction::DecrementRc`
    /// operating on an array directly from a `Instruction::MakeArray` or an
    /// intrinsic known to return a fresh array.
    fn is_inc_dec_instruction_on_known_array(
        instruction: &Instruction,
        dfg: &DataFlowGraph,
    ) -> bool {
        use Instruction::*;
        if let IncrementRc { value } | DecrementRc { value, .. } = instruction {
            dbg!(value);
            let Some(instruction) = dfg.get_local_or_global_instruction(*value) else {
                return false;
            };
            return match instruction {
                MakeArray { .. } => true,
                Call { func, .. } => {
                    matches!(&dfg[*func], Value::Intrinsic(_) | Value::ForeignFunction(_))
                }
                _ => false,
            };
        }
        false
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
                Instruction::ArrayGet { array, index, .. }
                | Instruction::ArraySet { array, index, .. } => (array, index),
                _ => panic!("Expected an ArrayGet or ArraySet instruction here"),
            };

            let call_stack = function.dfg.get_instruction_call_stack_id(instruction_id);

            let (lhs, rhs) = if function.dfg.get_numeric_constant(*index).is_some() {
                // If we are here it means the index is known but out of bounds. That's always an error!
                let false_const = function.dfg.make_constant(false.into(), NumericType::bool());
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (false_const, true_const)
            } else {
                let array_typ = function.dfg.type_of_value(*array);
                let element_size = array_typ.element_size() as u32;
                let len = match array_typ {
                    Type::Array(_, len) => len,
                    _ => panic!("Expected an array"),
                };
                // `index` will be relative to the flattened array length, so we need to take that into account
                let array_length = element_size * len;
                // let array_length = function.dfg.type_of_value(*array).flattened_size();

                // If we are here it means the index is dynamic, so let's add a check that it's less than length
                let length_type = NumericType::length_type();
                let index = function.dfg.insert_instruction_and_results(
                    Instruction::Cast(*index, length_type),
                    block_id,
                    None,
                    call_stack,
                );
                let index = index.first();
                let array_length =
                    function.dfg.make_constant((array_length as u128).into(), length_type);

                let is_index_out_of_bounds = function.dfg.insert_instruction_and_results(
                    Instruction::binary(BinaryOp::Lt, index, array_length),
                    block_id,
                    None,
                    call_stack,
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
                call_stack,
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
}

fn can_be_eliminated_if_unused(
    instruction: &Instruction,
    function: &Function,
    flattened: bool,
) -> bool {
    use Instruction::*;
    match instruction {
        Binary(binary) => {
            if matches!(binary.operator, BinaryOp::Div | BinaryOp::Mod) {
                if let Some(rhs) = function.dfg.get_numeric_constant(binary.rhs) {
                    rhs != FieldElement::zero()
                } else {
                    false
                }
            } else {
                // Checked binary operations can have different behavior depending on the predicate.
                !instruction.requires_acir_gen_predicate(&function.dfg)
            }
        }

        Cast(_, _)
        | Not(_)
        | Truncate { .. }
        | Allocate
        | Load { .. }
        | IfElse { .. }
        // Arrays are not side-effectual in Brillig where OOB checks are laid down explicitly in SSA.
        // However, arrays are side-effectual in ACIR (array OOB checks). 
        // We mark them available for deletion, but it is expected that this pass will insert
        // back the relevant side effects for array access in ACIR that can possible fail (e.g., index OOB or dynamic index).
        | ArrayGet { .. }
        | ArraySet { .. }
        | Noop
        | MakeArray { .. } => true,

        Store { .. } => should_remove_store(function, flattened),

        Constrain(..)
        | ConstrainNotEqual(..)
        | EnableSideEffectsIf { .. }
        | IncrementRc { .. }
        | DecrementRc { .. }
        | RangeCheck { .. } => false,

        // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
        Call { func, .. } => match function.dfg[*func] {
            // Explicitly allows removal of unused ec operations, even if they can fail
            Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul))
            | Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::EmbeddedCurveAdd)) => true,

            Value::Intrinsic(intrinsic) => !intrinsic.has_side_effects(),

            // All foreign functions are treated as having side effects.
            // This is because they can be used to pass information
            // from the ACVM to the external world during execution.
            Value::ForeignFunction(_) => false,

            // We use purity to determine whether functions contain side effects.
            // If we have an impure function, we cannot remove it even if it is unused.
            Value::Function(function_id) => match function.dfg.purity_of(function_id) {
                Some(Purity::Pure) => true,
                Some(Purity::PureWithPredicate) => false,
                Some(Purity::Impure) => false,
                None => false,
            },

            _ => false,
        },
    }
}

#[derive(Default)]
/// Per block RC tracker.
struct RcTracker {
    // We can track IncrementRc instructions per block to determine whether they are useless.
    // IncrementRc and DecrementRc instructions are normally side effectual instructions, but we remove
    // them if their value is not used anywhere in the function. However, even when their value is used, their existence
    // is pointless logic if there is no array set between the increment and the decrement of the reference counter.
    // We track per block whether an IncrementRc instruction has a paired DecrementRc instruction
    // with the same value but no array set in between.
    // If we see an inc/dec RC pair within a block we can safely remove both instructions.
    rcs_with_possible_pairs: HashMap<Type, Vec<RcInstruction>>,
    // Tracks repeated RC instructions: if there are two `inc_rc` for the same value in a row, the 2nd one is redundant.
    rc_pairs_to_remove: HashSet<InstructionId>,
    // We also separately track all IncrementRc instructions and all array types which have been mutably borrowed.
    // If an array is the same type as one of those non-mutated array types, we can safely remove all IncrementRc instructions on that array.
    inc_rcs: HashMap<ValueId, HashSet<InstructionId>>,
    // Mutated arrays shared across the blocks of the function.
    // When tracking mutations we consider arrays with the same type as all being possibly mutated.
    mutated_array_types: HashSet<Type>,
    // The SSA often creates patterns where after simplifications we end up with repeat
    // IncrementRc instructions on the same value. We track whether the previous instruction was an IncrementRc,
    // and if the current instruction is also an IncrementRc on the same value we remove the current instruction.
    // `None` if the previous instruction was anything other than an IncrementRc
    previous_inc_rc: Option<ValueId>,
}

impl RcTracker {
    fn new_block(&mut self) {
        self.rcs_with_possible_pairs.clear();
        self.rc_pairs_to_remove.clear();
        self.inc_rcs.clear();
        self.previous_inc_rc = Default::default();
    }

    fn mark_terminator_arrays_as_used(&mut self, function: &Function, block: &BasicBlock) {
        block.unwrap_terminator().for_each_value(|value| {
            self.handle_value_for_mutated_array_types(value, &function.dfg);
        });
    }

    fn handle_value_for_mutated_array_types(&mut self, value: ValueId, dfg: &DataFlowGraph) {
        let typ = dfg.type_of_value(value);
        if !matches!(&typ, Type::Array(_, _) | Type::Slice(_)) {
            return;
        }

        self.mutated_array_types.insert(typ);

        if dfg.is_global(value) {
            return;
        }

        // Also check if the value is a MakeArray instruction. If so, do the same check for all of its values.
        let Value::Instruction { instruction, .. } = &dfg[value] else {
            return;
        };
        let Instruction::MakeArray { elements, typ: _ } = &dfg[*instruction] else {
            return;
        };

        for element in elements {
            self.handle_value_for_mutated_array_types(*element, dfg);
        }
    }

    fn track_inc_rcs_to_remove(&mut self, instruction_id: InstructionId, function: &Function) {
        let instruction = &function.dfg[instruction_id];

        // Deduplicate IncRC instructions.
        if let Instruction::IncrementRc { value } = instruction {
            if let Some(previous_value) = self.previous_inc_rc {
                if previous_value == *value {
                    self.rc_pairs_to_remove.insert(instruction_id);
                }
            }
            self.previous_inc_rc = Some(*value);
        } else {
            // Reset the deduplication.
            self.previous_inc_rc = None;
        }

        // DIE loops over a block in reverse order, so we insert an RC instruction for possible removal
        // when we see a DecrementRc and check whether it was possibly mutated when we see an IncrementRc.
        match instruction {
            Instruction::IncrementRc { value } => {
                // Get any RC instruction recorded further down the block for this array;
                // if it exists and not marked as mutated, then both RCs can be removed.
                if let Some(inc_rc) =
                    pop_rc_for(*value, function, &mut self.rcs_with_possible_pairs)
                {
                    if !inc_rc.possibly_mutated {
                        self.rc_pairs_to_remove.insert(inc_rc.id);
                        self.rc_pairs_to_remove.insert(instruction_id);
                    }
                }
                // Remember that this array was RC'd by this instruction.
                self.inc_rcs.entry(*value).or_default().insert(instruction_id);
            }
            Instruction::DecrementRc { value, .. } => {
                let typ = function.dfg.type_of_value(*value);

                // We assume arrays aren't mutated until we find an array_set
                let dec_rc =
                    RcInstruction { id: instruction_id, array: *value, possibly_mutated: false };
                self.rcs_with_possible_pairs.entry(typ).or_default().push(dec_rc);
            }
            Instruction::ArraySet { array, value, .. } => {
                let typ = function.dfg.type_of_value(*array);
                // We mark all RCs that refer to arrays with a matching type as the one being set, as possibly mutated.
                if let Some(dec_rcs) = self.rcs_with_possible_pairs.get_mut(&typ) {
                    for dec_rc in dec_rcs {
                        dec_rc.possibly_mutated = true;
                    }
                }
                self.mutated_array_types.insert(typ);

                let value_typ = function.dfg.type_of_value(*value);
                if value_typ.is_array() {
                    self.mutated_array_types.insert(value_typ);
                }
            }
            Instruction::Store { value, .. } => {
                // We are very conservative and say that any store of an array type means it has the potential to be mutated.
                self.handle_value_for_mutated_array_types(*value, &function.dfg);
            }
            Instruction::Call { arguments, .. } => {
                // Treat any array-type arguments to calls as possible sources of mutation.
                // During the preprocessing of functions in isolation we don't want to
                // get rid of IncRCs arrays that can potentially be mutated outside.
                for arg in arguments {
                    self.handle_value_for_mutated_array_types(*arg, &function.dfg);
                }
            }
            _ => {}
        }
    }

    /// Get all RC instructions which work on arrays whose type has not been marked as mutated.
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

fn instruction_might_result_in_out_of_bounds(
    function: &Function,
    instruction: &Instruction,
) -> bool {
    use Instruction::*;
    match instruction {
        ArrayGet { array, index, .. } | ArraySet { array, index, .. } => {
            // We only care about arrays here as slices are expected to have explicit checks laid down in the initial SSA.
            function.dfg.try_get_array_length(*array).is_some()
                && !function.dfg.is_safe_index(*index, *array)
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
    call_stack: CallStackId,
) -> (ValueId, ValueId) {
    // See if there's an active "enable side effects" condition
    let Some(condition) = side_effects_condition else {
        return (lhs, rhs);
    };

    let dfg = &mut function.dfg;

    // Condition needs to be cast to argument type in order to multiply them together.
    // In our case, lhs is always a boolean.
    let cast = Instruction::Cast(condition, NumericType::bool());
    let casted_condition = dfg.insert_instruction_and_results(cast, block_id, None, call_stack);
    let casted_condition = casted_condition.first();

    // Unchecked mul because the side effects var is always 0 or 1
    let lhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul { unchecked: true }, lhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let lhs = lhs.first();

    // Unchecked mul because the side effects var is always 0 or 1
    let rhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul { unchecked: true }, rhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let rhs = rhs.first();

    (lhs, rhs)
}

/// Store instructions must be removed by DIE in acir code, any load
/// instructions should already be unused by that point.
///
/// Note that this check assumes that it is being performed after the flattening
/// pass and after the last mem2reg pass. This is currently the case for the DIE
/// pass where this check is done, but does mean that we cannot perform mem2reg
/// after the DIE pass.
fn should_remove_store(func: &Function, flattened: bool) -> bool {
    flattened && func.runtime().is_acir() && func.reachable_blocks().len() == 1
}

/// Check post-execution properties:
/// * Store and Load instructions should be removed from ACIR after flattening.
#[cfg(debug_assertions)]
fn die_post_check(func: &Function, flattened: bool) {
    if should_remove_store(func, flattened) {
        for block_id in func.reachable_blocks() {
            for (i, instruction_id) in func.dfg[block_id].instructions().iter().enumerate() {
                let instruction = &func.dfg[*instruction_id];
                if matches!(instruction, Instruction::Load { .. } | Instruction::Store { .. }) {
                    panic!(
                        "not expected to have Load or Store instruction after DIE in an ACIR function: {} {} / {block_id} / {i}: {:?}",
                        func.name(),
                        func.id(),
                        instruction
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use im::vector;
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa,
            function_builder::FunctionBuilder,
            ir::{
                function::RuntimeType,
                instruction::ArrayOffset,
                map::Id,
                types::{NumericType, Type},
            },
            opt::assert_normalized_ssa_equals,
        },
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
                v11 = add v1, Field 2
                v13 = add v9, Field 3
                v14 = add v13, v13
                call assert_constant(v10)
                return v11
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v3 = add v0, Field 2
            jmp b1(v3)
          b1(v1: Field):
            v4 = allocate -> &mut Field
            store Field 1 at v4
            v6 = load v4 -> Field
            v7 = add v6, Field 1
            v8 = add v1, Field 2
            call assert_constant(v7)
            return v8
        }
        ");
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
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ");
    }

    #[test]
    fn remove_useless_paired_rcs_even_when_used() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v2 = array_get v0, index u32 0 -> Field
                dec_rc v0
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        ");
    }

    #[test]
    fn keep_paired_rcs_with_array_set() {
        let src = "
            brillig(inline) fn main f0 {
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
        // brillig(inline) fn main f0 {
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
        builder.set_runtime(RuntimeType::Brillig(InlineType::Inline));
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
        let mutable = false;
        let offset = ArrayOffset::None;
        let v5 = builder.insert_array_set(v3, zero, one, mutable, offset);
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
    fn keep_inc_rc_on_borrowed_array_set() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [u32; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value u32 1
            inc_rc v0
            inc_rc v0
            inc_rc v0
            v4 = array_get v3, index u32 1 -> u32
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        // We expect the output to be unchanged
        // Except for the repeated inc_rc instructions
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [u32; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value u32 1
            inc_rc v0
            v4 = array_get v3, index u32 1 -> u32
            return v4
        }
        ");
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

    #[test]
    fn does_not_remove_inc_rcs_that_are_never_mutably_borrowed() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            inc_rc v0
            inc_rc v0
            v2 = array_get v0, index u32 0 -> Field
            inc_rc v0
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        // The instruction count never includes the terminator instruction
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 5);

        let ssa = ssa.dead_instruction_elimination();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v2 = array_get v0, index u32 0 -> Field
            inc_rc v0
            return v2
        }
        ");
    }

    #[test]
    fn do_not_remove_inc_rcs_for_arrays_in_terminator() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            inc_rc v0
            inc_rc v0
            v2 = array_get v0, index u32 0 -> Field
            inc_rc v0
            return v0, v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.dead_instruction_elimination();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v2 = array_get v0, index u32 0 -> Field
            inc_rc v0
            return v0, v2
        }
        ");
    }

    #[test]
    fn do_not_remove_inc_rc_if_used_as_call_arg() {
        // We do not want to remove inc_rc instructions on values
        // that are passed as call arguments.
        //
        // We could have previously inlined a function which does the following:
        // - Accepts a mutable array as an argument
        // - Writes to that array
        // - Passes the new array to another call
        //
        // It is possible then that the mutation gets simplified out after inlining.
        // If we then remove the inc_rc as we see no mutations to that array in the block,
        // we may end up with an the incorrect reference count.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v4 = make_array [Field 0, Field 1, Field 2] : [Field; 3]
            inc_rc v4
            v6 = call f1(v4) -> Field
            constrain v0 == v6
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: [Field; 3]):
            return Field 1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_remove_mutable_reference_params() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = allocate -> &mut Field
            store v0 at v2
            call f1(v2)
            v4 = load v2 -> Field
            v5 = eq v4, v1
            constrain v4 == v1
            return
        }
        acir(inline) fn Add10 f1 {
          b0(v0: &mut Field):
            v1 = load v0 -> Field
            v2 = load v0 -> Field
            v4 = add v2, Field 10
            store v4 at v0
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // Even though these ACIR functions only have 1 block, we have not inlined and flattened anything yet.
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = allocate -> &mut Field
            store v0 at v2
            call f1(v2)
            v4 = load v2 -> Field
            constrain v4 == v1
            return
        }
        acir(inline) fn Add10 f1 {
          b0(v0: &mut Field):
            v1 = load v0 -> Field
            v3 = add v1, Field 10
            store v3 at v0
            return
        }
        ");
    }

    #[test]
    fn do_not_remove_inc_rc_if_mutated_in_other_block() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut [Field; 3]):
            v1 = load v0 -> [Field; 3]
            inc_rc v1
            jmp b1()
          b1():
            v2 = load v0 -> [Field; 3]
            v3 = array_set v2, index u32 0, value u32 0
            store v3 at v0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: &mut [Field; 3]):
            v1 = load v0 -> [Field; 3]
            inc_rc v1
            jmp b1()
          b1():
            v2 = load v0 -> [Field; 3]
            v4 = array_set v2, index u32 0, value u32 0
            store v4 at v0
            return
        }
        ");
    }

    #[test]
    fn remove_dead_pure_function_call() {
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        acir(inline) fn pure_basic f1 {
          b0():
            v2 = allocate -> &mut Field
            store Field 0 at v2
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        // We expect the call to f1 in f0 to be removed
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) pure fn main f0 {
          b0():
            return
        }
        acir(inline) pure fn pure_basic f1 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            return
        }
        "#);
    }

    #[test]
    fn do_not_remove_impure_function_call() {
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            call f1(v0)
            return
        }
        acir(inline) fn impure_take_ref f1 {
          b0(v0: &mut Field):
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        // We expect the program to be unchanged except that functions are labeled with purities now
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            call f1(v0)
            return
        }
        acir(inline) impure fn impure_take_ref f1 {
          b0(v0: &mut Field):
            return
        }
        "#);
    }

    #[test]
    fn do_not_remove_pure_with_predicates_function_call() {
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(Field 0)
            return
        }
        acir(inline) fn predicate_constrain f1 {
          b0(v0: Field):
            constrain v0 == Field 0
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        // We expect the program to be unchanged except that functions are labeled with purities now
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            call f1(Field 0)
            return
        }
        acir(inline) predicate_pure fn predicate_constrain f1 {
          b0(v0: Field):
            constrain v0 == Field 0
            return
        }
        "#);
    }

    #[test]
    fn does_not_remove_inc_rc_of_return_value_that_points_to_a_make_array() {
        // Here we would previously incorrectly remove `inc_rc v1`
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v1 = make_array [u1 1] : [u1; 1]
            v2 = make_array [v1] : [[u1; 1]; 1]
            inc_rc v1
            inc_rc v2 
            return v2
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_crash_for_value_pointing_to_make_array_pointing_to_global() {
        let src = r#"
        g0 = make_array [u1 1] : [u1; 1]

        brillig(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [g0] : [[u1; 1]; 1]
            inc_rc v0
            return v0
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn replace_out_of_bounds_array_get_with_failing_constrain() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 1]):
            v1 = array_get v0, index u32 2 -> Field
            return v0
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 1]):
            constrain u1 0 == u1 1, "Index out of bounds"
            return v0
        }
        "#);
    }

    #[test]
    fn replace_out_of_bounds_array_set_with_failing_constrain() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 1]):
            v1 = array_set v0, index u32 2, value Field 0
            return v0
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 1]):
            constrain u1 0 == u1 1, "Index out of bounds"
            return v0
        }
        "#);
    }

    #[test]
    fn does_not_replace_valid_array_set() {
        let src = r"
        acir(inline) fn main f0 {
          b0(v0: [u8; 32]):                 	
            v3 = array_set v0, index u32 0, value u8 5    	                     	
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination_pre_flattening();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn removes_an_array_get_which_is_in_bounds_due_to_offset() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_get v0, index u32 1 minus 1 -> Field
            return v0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            return v0
        }
        ");
    }

    #[test]
    fn correctly_handles_chains_of_array_gets() {
        //! This test checks that if there's a chain of `array_get` instructions which use the result of the previous
        //! read as the index of the next `array_get`, we only replace the final `array_get` and do not propagate
        //! up the chain. Otherwise we remove instructions for which the instructions are still used.
        // SSA generated from `compile_success_empty/regression_7785` (slightly modified)
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = make_array [u32 0, u32 0] : [u32; 2]
            v3 = array_get v2, index v0 -> u32
            v4 = array_get v2, index v3 -> u32
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        // Previously this would produce the SSA:
        //
        // acir(inline) predicate_pure fn main f0 {
        //   b0():
        //     v1 = call f1() -> u32
        //     v3 = lt v1, u32 2
        //     constrain v3 == u1 1, "Index out of bounds"
        //     v5 = lt v4, u32 2  <-- Notice that `v4` has now been orphaned
        //     constrain v5 == u1 1, "Index out of bounds"
        //     return
        //   }
        // brillig(inline) predicate_pure fn inject_value f1 {
        //   b0():
        //     return u32 0
        // }
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = make_array [u32 0, u32 0] : [u32; 2]
            v3 = array_get v2, index v0 -> u32
            v5 = lt v3, u32 2
            constrain v5 == u1 1, "Index out of bounds"
            return
        }
        "#);
    }

    #[test]
    fn do_not_remove_inc_rc_on_nested_constant_array() {
        let src = "
        brillig(inline) fn func_1 f0 {
          b0(v0: [[Field; 1]; 1]):
            v1 = allocate -> &mut [[Field; 1]; 1]
            store v0 at v1
            v3 = make_array [Field 2] : [Field; 1]
            v4 = allocate -> &mut Field
            store Field 0 at v4
            jmp b1()
          b1():
            v6 = load v4 -> Field
            v7 = eq v6, Field 2
            jmpif v7 then: b2, else: b3
          b2():
            jmp b4()
          b3():
            v8 = load v4 -> Field
            v10 = add v8, Field 1
            store v10 at v4
            v11 = allocate -> &mut Field
            store Field 0 at v11
            jmp b5()
          b4():
            v23 = load v1 -> [[Field; 1]; 1]
            v24 = array_get v23, index u32 0 -> [Field; 1]
            v25 = array_get v24, index u32 0 -> Field
            return v25
          b5():
            v12 = load v11 -> Field
            v13 = eq v12, Field 1
            jmpif v13 then: b6, else: b7
          b6():
            jmp b8()
          b7():
            v14 = load v11 -> Field
            v15 = add v14, Field 1
            store v15 at v11
            v16 = load v1 -> [[Field; 1]; 1]
            v18 = array_get v16, index u32 0 -> [Field; 1]
            v19 = array_set v18, index u32 0, value Field 1
            v20 = array_set v16, index u32 0, value v19
            store v20 at v1
            jmp b9()
          b8():
            inc_rc v3
            v21 = load v1 -> [[Field; 1]; 1]
            v22 = array_set v21, index u32 0, value v3
            store v22 at v1
            jmp b10()
          b9():
            jmp b5()
          b10():
            jmp b1()
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        // If `inc_rc v3` were removed, we risk it later being mutated in `v19 = array_set v18, index u32 0, value Field 1`.
        // Thus, when we later go to do `v22 = array_set v21, index u32 0, value v3` once more, we will be writing [1] rather than [2].
        assert_normalized_ssa_equals(ssa, src);
    }
}
