//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
//!
//! DIE also tracks which block parameters are unused.
//! Unused parameters are then pruned by the [prune_dead_parameters] pass.
//!
//! ## Design
//! - Instructions are scanned in reverse (within each block), keeping track of
//!   used values. If the current instruction is safe for removal (no side effects)
//!   and its results are all unused the instruction will be marked for removal.
//!   Traversing in reverse enables removing entire unused chains of computation.
//! - The pass also tracks unused [IncrementRc][Instruction::IncrementRc] and [DecrementRc][Instruction::DecrementRc] instructions.
//!   As these instructions contain side effects we only remove them after analyzing an entire function to see if their values are unused.
//! - Block parameters are also tracked. Unused parameters are pruned in a follow-up [prune_dead_parameters] pass
//!   to maintain separation of concerns and SSA consistency.
//! - The main DIE pass and dead parameter pruning are called in a fixed point feedback loop that stops
//!   once there are no more unused parameters.
//!
//! ## Runtime Differences
//! - ACIR
//!   - Array operations implicitly enforce OOB checks. DIE therefore
//!     inserts synthetic OOB checks if array ops are removed, to preserve side-effect
//!     ordering semantics.
//!     As the SSA flattens all tuples, unused accesses on composite arrays can lead to potentially
//!     multiple unused array accesses. As to avoid redundant OOB checks, we search for "array get groups"
//!     and only insert a single OOB check for an array get group.
//!   - [Store][Instruction::Store] instructions can only be removed if the `flattened` flag is set.
//!   - Instructions that create the value which is returned in the databus (if present) is not removed.
//! - Brillig
//!   - Array operations are explicit and thus it is expected separate OOB checks
//!     have been laid down. Thus, no extra instructions are inserted for unused array accesses.
//!   - [Store][Instruction::Store] instructions are never removed.
//!   - The databus is never used to return values, so instructions to create a Field array to return are never generated.
//!
//! ## Preconditions
//! - ACIR: By default the pass must be run after [mem2reg][crate::ssa::opt::mem2reg] and [CFG flattening][crate::ssa::opt::flatten_cfg].
//!   If the pass is run before these passes, it must be explicitly stated.
//! - Must be run on the full [Ssa], not individual [Function]s, to avoid dangling
//!   parameter references from dead parameter pruning.
//!
//! ## Post-conditions
//! - If DIE was run after mem2reg and flattening, no unreachable
//!   [Load][Instruction::Load] or [Store][Instruction::Store] instructions should remain in ACIR code.
//! - All unused SSA instructions (pure ops, unused RCs, dead params) are removed.
use acvm::{AcirField, FieldElement};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        integer::IntegerConstant,
        post_order::PostOrder,
        types::NumericType,
        value::{Value, ValueId},
    },
    opt::{die::array_oob_checks::should_insert_oob_check, pure::Purity},
    ssa_gen::Ssa,
};

mod array_oob_checks;
mod prune_dead_parameters;

impl Ssa {
    /// Performs Dead Instruction Elimination (DIE) to remove any instructions with
    /// unused results.
    ///
    /// This step should come after the flattening of the CFG and mem2reg.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn dead_instruction_elimination(self) -> Ssa {
        self.dead_instruction_elimination_with_pruning(true)
    }

    /// The elimination of certain unused instructions assumes that the DIE pass runs after
    /// the flattening of the CFG, but if that's not the case then we should not eliminate
    /// them just yet.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn dead_instruction_elimination_pre_flattening(self) -> Ssa {
        self.dead_instruction_elimination_with_pruning(false)
    }

    fn dead_instruction_elimination_with_pruning(mut self, flattened: bool) -> Ssa {
        #[cfg(debug_assertions)]
        self.functions.values().for_each(|func| die_pre_check(func, flattened));

        let mut previous_unused_params = None;
        loop {
            let (new_ssa, result) = self.dead_instruction_elimination_inner(flattened);

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

    fn dead_instruction_elimination_inner(mut self, flattened: bool) -> (Ssa, DIEResult) {
        let result = self
            .functions
            .par_iter_mut()
            .map(|(id, func)| {
                let unused_params = func.dead_instruction_elimination(true, flattened);
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
    ) -> HashMap<BasicBlockId, Vec<ValueId>> {
        let mut context = Context { flattened, ..Default::default() };

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
            return self.dead_instruction_elimination(false, flattened);
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

        // Indices of instructions that might be out of bounds.
        // We'll remove those, but before that we'll insert bounds checks for them.
        let mut possible_index_out_of_bounds_indices = Vec::new();

        // Going in reverse so we know if a result of an instruction was used.
        for (instruction_index, instruction_id) in block.instructions().iter().enumerate().rev() {
            let instruction = &function.dfg[*instruction_id];

            if self.is_unused(*instruction_id, function) {
                self.instructions_to_remove.insert(*instruction_id);

                if insert_out_of_bounds_checks && should_insert_oob_check(function, instruction) {
                    possible_index_out_of_bounds_indices.push(instruction_index);
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
                    self.rc_instructions.push((*instruction_id, block_id));
                } else {
                    instruction.for_each_value(|value| {
                        self.mark_used_instruction_results(&function.dfg, value);
                    });
                }
            }
        }

        // If there are some instructions that might trigger an out of bounds error,
        // first add constrain checks. Then run the DIE pass again, which will remove those
        // but leave the constrains (any any value needed by those constrains)
        if !possible_index_out_of_bounds_indices.is_empty() {
            let inserted_check = self.replace_array_instructions_with_out_of_bounds_checks(
                function,
                block_id,
                &mut possible_index_out_of_bounds_indices,
            );
            // There's a chance we didn't insert any checks, so we could proceed with DIE.
            // This can happen for example with arrays of a complex type, where one part
            // of the complex type is used, while the other is not, in which case no constraint
            // is inserted, because the use itself will cause an OOB error.
            // By proceeding, the unused access will be removed.
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
            let results_unused = results.iter().all(|result| !self.used_values.contains(result));
            results_unused && !function.dfg.is_returned_in_databus(instruction_id)
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
                    // Division by zero must remain
                    if rhs == FieldElement::zero() {
                        return false;
                    }

                    // There's one more case: signed division that does MIN / -1
                    let typ = function.dfg.type_of_value(binary.lhs).unwrap_numeric();
                    if let NumericType::Signed { bit_size } = typ {
                        if let Some(rhs) = IntegerConstant::from_numeric_constant(rhs, typ) {
                            let minus_one = IntegerConstant::Signed { value: -1, bit_size };
                            if rhs == minus_one {
                                return false;
                            }
                        }
                    }

                    true
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

/// Check pre-execution properties:
/// * Passing `flattened = true` will confirm the CFG has already been flattened into a single block for ACIR functions
#[cfg(debug_assertions)]
fn die_pre_check(func: &Function, flattened: bool) {
    if flattened {
        super::flatten_cfg::flatten_cfg_post_check(func);
    }
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
                map::Id,
                types::{NumericType, Type},
            },
            opt::assert_ssa_does_not_change,
        },
    };
    use test_case::test_case;

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
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false);

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
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
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
        let v5 = builder.insert_array_set(v3, zero, one, mutable);
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
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
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
            v1 = array_get v0, index u32 0 -> Field
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
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
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false);

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
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false);

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
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false);

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
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false);

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
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
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
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
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
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination_pre_flattening);
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
            v4 = cast v3 as u64
            v6 = lt v4, u64 2
            constrain v6 == u1 1, "Index out of bounds"
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
        // If `inc_rc v3` were removed, we risk it later being mutated in `v19 = array_set v18, index u32 0, value Field 1`.
        // Thus, when we later go to do `v22 = array_set v21, index u32 0, value v3` once more, we will be writing [1] rather than [2].
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
    }

    #[test]
    fn replaces_oob_followed_by_safe_access_with_constraint() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array b"KO"
            v4 = make_array [u1 0, v2] : [(u1, [u8; 2]); 1]
            v6 = array_get v4, index u32 20 -> u1
            v8 = array_get v4, index u32 1 -> [u8; 2]
            return v8
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array b"KO"
            v4 = make_array [u1 0, v2] : [(u1, [u8; 2]); 1]
            constrain u1 0 == u1 1, "Index out of bounds"
            v7 = array_get v4, index u32 1 -> [u8; 2]
            return v7
        }
        "#);
    }

    #[test]
    fn keeps_unused_databus_return_value() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          return_data: v0
          b0():
            v0 = make_array [Field 0] : [Field; 1]
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
    }

    #[test_case("ecdsa_secp256k1")]
    #[test_case("ecdsa_secp256r1")]
    fn does_not_remove_unused_ecdsa_verification(ecdsa_func: &str) {
        let src = format!(
            r#"
        acir(inline) fn main f0 {{
            b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
            v4 = call {ecdsa_func}(v0, v1, v2, v3, u1 1) -> u1
            return
        }}
        "#
        );
        assert_ssa_does_not_change(&src, Ssa::dead_instruction_elimination);
    }

    #[test]
    fn does_not_remove_unused_curve_operations() {
        let src = r#"
        acir(inline) fn main f0 {{
            b0(v0: Field, v1: Field, v2: Field):
            v6 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 1]
            v8 = make_array [v0, Field 0] : [(Field, Field); 1]
            v11 = call multi_scalar_mul(v6, v8, u1 1) -> [(Field, Field, u1); 1]
            v12 = call embedded_curve_add(v0, v1, u1 0, v2, Field 3, u1 0, u1 0) -> [(Field, Field, u1); 1]
            return
        }}
        "#;

        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
    }

    #[test]
    fn removes_unused_known_small_bit_shifts() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = shl v0, u32 2
            v2 = shr v0, u32 3
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            return
        }
        ");
    }

    #[test]
    fn keeps_bit_shifts_by_unknown_amounts() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = shl v0, v1
            return
        }
        "#;

        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
    }

    #[test]
    fn keeps_bit_shifts_by_known_large_amounts() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = shl v0, u32 33
            return
        }
        "#;

        assert_ssa_does_not_change(src, Ssa::dead_instruction_elimination);
    }
}
