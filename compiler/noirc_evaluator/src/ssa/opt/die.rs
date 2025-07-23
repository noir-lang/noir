//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
//!
//! DIE also tracks which block parameters are unused.
//! Unused parameters are then pruned by the [prune_dead_parameters] pass.
use acvm::{AcirField, acir::BlackBoxFunc};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use num_bigint::BigInt;
use num_traits::Zero;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
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
        // Perform post-checks on the SSA.
        let check = |ssa: Ssa| {
            // Check that we have established the properties expected from this pass.
            #[cfg(debug_assertions)]
            ssa.functions.values().for_each(|f| die_post_check(f, flattened));
            ssa
        };

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
                return check(new_ssa);
            }

            if let Some(previous) = &previous_unused_params {
                // If no changes to dead parameters occurred, return early
                if previous == &result.unused_parameters {
                    return check(new_ssa);
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
                let unused_params = func.dead_instruction_elimination(flattened, skip_brillig);
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

        let blocks = PostOrder::with_function(self);
        let mut unused_params_per_block = HashMap::default();
        for block in blocks.as_slice() {
            context.remove_unused_instructions_in_block(self, *block);

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
    fn remove_unused_instructions_in_block(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
    ) {
        let block = &function.dfg[block_id];
        self.mark_terminator_values_as_used(function, block);

        self.rc_tracker.new_block();
        self.rc_tracker.mark_terminator_arrays_as_used(function, block);

        // Going in reverse so we know if a result of an instruction was used.
        for instruction_id in block.instructions().iter().rev() {
            let instruction = &function.dfg[*instruction_id];

            if self.is_unused(*instruction_id, function) {
                self.instructions_to_remove.insert(*instruction_id);
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

            self.rc_tracker.track_inc_rcs_to_remove(*instruction_id, function);
        }

        self.instructions_to_remove.extend(self.rc_tracker.get_non_mutated_arrays(&function.dfg));
        self.instructions_to_remove.extend(self.rc_tracker.rc_pairs_to_remove.drain());

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));
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
                    rhs != BigInt::zero()
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
        | ArrayGet { .. }
        | IfElse { .. }
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
            let typ = function.dfg.type_of_value(value);
            if matches!(&typ, Type::Array(_, _) | Type::Slice(_)) {
                self.mutated_array_types.insert(typ);
            }
        });
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
            Instruction::ArraySet { array, .. } => {
                let typ = function.dfg.type_of_value(*array);
                // We mark all RCs that refer to arrays with a matching type as the one being set, as possibly mutated.
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
                // Treat any array-type arguments to calls as possible sources of mutation.
                // During the preprocessing of functions in isolation we don't want to
                // get rid of IncRCs arrays that can potentially be mutated outside.
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
}
