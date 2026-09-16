//! Predicates a trivial conditional in place instead of leaving it for flattening.
//!
//! A guarded assertion compiles to a two-block diamond per source-level `if`:
//!
//! ```text
//! b0():
//!   jmpif v3 then: b1(), else: b2()
//! b1():
//!   v4 = array_get v0, index u32 7 -> Field
//!   constrain v4 == Field 0, "..."
//!   jmp b2()
//! b2():
//!   ...
//! ```
//!
//! Flattening eventually collapses that into the entry block, replacing each `constrain lhs == rhs`
//! with `constrain condition * lhs == condition * rhs`. This pass does the same thing for the
//! diamonds simple enough to handle locally, so they never reach the intervening passes.
//!
//! The shape is worth handling early because it multiplies: a loop that guards an assertion over
//! an `N`-element array unrolls into `N` diamonds, so for a large array they can dominate the
//! block count of the whole function, and every pass between unrolling and flattening walks them.
//!
//! The rewrite is only applied when the guarded block is trivial:
//!   - the conditional is a diamond: the `else` target is the block the `then` branch joins, the
//!     guarded block is reached only from the conditional, and neither takes block parameters;
//!   - every instruction in it either needs no ACIR predicate, so evaluating it unconditionally
//!     cannot fail, or is a `Constrain`, which gets the same multiply flattening would give it.
//!
//! Anything else - a store, a range check, a call, an `array_set`, arithmetic that can overflow, an
//! `array_get` whose index is not provably in bounds - is left for flattening, which knows how to
//! nullify each of them.
//!
//! # Ordering
//!
//! This must run after unrolling, which is what creates the repeated diamonds, and before
//! flattening, which is what it takes work away from. The pre-check pins both ends: no loops are
//! left to unroll, and no `enable_side_effects` exists yet, that being what flattening introduces.
//!
//! It is a no-op on Brillig functions. Their branches execute, so there is no predicate to weave
//! in, and `basic_conditional` already flattens the conditionals worth flattening there.

use noirc_errors::call_stack::CallStackId;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::{Function, RuntimeType},
        instruction::{BinaryOp, Instruction, InstructionId, TerminatorInstruction},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See the [module docs][self] for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn flatten_trivial_conditionals(mut self) -> Self {
        for function in self.functions.values_mut() {
            // Brillig executes branches, so there is no predicate to fold in.
            if matches!(function.runtime(), RuntimeType::Brillig(_)) {
                continue;
            }

            #[cfg(debug_assertions)]
            trivial_conditional_pre_check(function);

            function.flatten_trivial_conditionals();
        }
        self
    }
}

/// Pre-check condition for [`Function::flatten_trivial_conditionals`].
///
/// Panics if the function still has a loop to unroll, meaning the diamonds this looks for have not
/// been multiplied out yet, or already contains an `enable_side_effects`, meaning flattening has
/// run and they are gone. Together they hold the pass in place between the two.
#[cfg(debug_assertions)]
fn trivial_conditional_pre_check(function: &Function) {
    super::checks::assert_no_loops(function);
    super::checks::for_each_instruction(function, |instruction, _dfg| {
        super::checks::assert_not_enable_side_effects(instruction);
    });
}

/// A conditional whose guarded block can be predicated in place.
struct TrivialConditional {
    /// The block holding the `jmpif`, and where the guarded instructions end up.
    entry: BasicBlockId,
    /// The guarded block, emptied by the rewrite.
    guarded: BasicBlockId,
    /// The block both edges lead to, which `entry` jumps to unconditionally afterwards.
    join: BasicBlockId,
    condition: ValueId,
}

impl Function {
    fn flatten_trivial_conditionals(&mut self) {
        let cfg = ControlFlowGraph::with_function(self);
        let conditionals: Vec<_> = self
            .reachable_blocks()
            .into_iter()
            .filter_map(|block| self.trivial_conditional_at(block, &cfg))
            .collect();

        for conditional in conditionals {
            self.predicate_conditional(conditional);
        }
    }

    /// Recognises the diamond described in the [module docs][self] rooted at `block`.
    fn trivial_conditional_at(
        &self,
        block: BasicBlockId,
        cfg: &ControlFlowGraph,
    ) -> Option<TrivialConditional> {
        let TerminatorInstruction::JmpIf { condition, then_destination, else_destination, .. } =
            *self.dfg[block].terminator()?
        else {
            return None;
        };

        // The guarded block must join straight back to where the untaken branch goes, so that
        // skipping it is the same as not entering it.
        let TerminatorInstruction::Jmp { destination, arguments, .. } =
            self.dfg[then_destination].terminator()?
        else {
            return None;
        };
        if *destination != else_destination || !arguments.is_empty() {
            return None;
        }

        // Reached from anywhere else and the instructions cannot move to this entry block; taking
        // parameters and they would need merging rather than hoisting.
        if cfg.predecessors(then_destination).len() != 1
            || !self.dfg[then_destination].parameters().is_empty()
            || !self.dfg[else_destination].parameters().is_empty()
        {
            return None;
        }

        let trivial = self.dfg[then_destination]
            .instructions()
            .iter()
            .all(|instruction| self.can_predicate_in_place(*instruction));
        if !trivial {
            return None;
        }

        Some(TrivialConditional {
            entry: block,
            guarded: then_destination,
            join: else_destination,
            condition,
        })
    }

    /// Whether an instruction can be moved out of the guarded block, either because running it
    /// unconditionally cannot fail or because the predicate can be folded into it here.
    fn can_predicate_in_place(&self, instruction: InstructionId) -> bool {
        match &self.dfg[instruction] {
            // Flattening multiplies both sides by the condition; so does this pass.
            Instruction::Constrain(..) => true,
            // Everything else has to be safe to evaluate whether or not the branch is taken.
            // `requires_acir_gen_predicate` is exactly that question, but it says nothing about
            // instructions flattening rewrites rather than predicates, so list those out.
            Instruction::Binary(_)
            | Instruction::Cast(..)
            | Instruction::Not(_)
            | Instruction::Truncate { .. }
            | Instruction::ArrayGet { .. }
            | Instruction::MakeArray { .. }
            | Instruction::Noop => !self.dfg[instruction].requires_acir_gen_predicate(&self.dfg),
            Instruction::ConstrainNotEqual(..)
            | Instruction::RangeCheck { .. }
            | Instruction::Store { .. }
            | Instruction::Load { .. }
            | Instruction::Allocate
            | Instruction::Call { .. }
            | Instruction::ArraySet { .. }
            | Instruction::IfElse { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::EnableSideEffectsIf { .. } => false,
        }
    }

    /// Moves the guarded instructions into the entry block, predicating the assertions, and makes
    /// the entry block jump straight to the join.
    fn predicate_conditional(&mut self, conditional: TrivialConditional) {
        let TrivialConditional { entry, guarded, join, condition } = conditional;

        // One cast of the condition per type it is multiplied into, however many assertions the
        // block holds.
        let mut casts: HashMap<NumericType, ValueId> = HashMap::default();

        for instruction in self.dfg[guarded].take_instructions() {
            if let Instruction::Constrain(lhs, rhs, message) = self.dfg[instruction].clone() {
                let call_stack = self.dfg.get_instruction_call_stack_id(instruction);
                let lhs = self.mul_by_condition(lhs, condition, entry, call_stack, &mut casts);
                let rhs = self.mul_by_condition(rhs, condition, entry, call_stack, &mut casts);
                self.dfg[instruction] = Instruction::Constrain(lhs, rhs, message);
            }
            self.dfg[entry].insert_instruction(instruction);
        }

        let call_stack = self.dfg[entry]
            .terminator()
            .map(|terminator| terminator.call_stack())
            .unwrap_or_default();
        self.dfg[entry].set_terminator(TerminatorInstruction::Jmp {
            destination: join,
            arguments: Vec::new(),
            call_stack,
        });
    }

    /// `value * condition`, so that a constraint holds trivially when the branch is not taken.
    fn mul_by_condition(
        &mut self,
        value: ValueId,
        condition: ValueId,
        block: BasicBlockId,
        call_stack: CallStackId,
        casts: &mut HashMap<NumericType, ValueId>,
    ) -> ValueId {
        let numeric_type = self.dfg.type_of_value(value).unwrap_numeric();
        let condition = match casts.get(&numeric_type) {
            Some(condition) => *condition,
            None => {
                let cast = self
                    .dfg
                    .insert_instruction_and_results(
                        Instruction::Cast(condition, numeric_type),
                        block,
                        None,
                        call_stack,
                    )
                    .first();
                casts.insert(numeric_type, cast);
                cast
            }
        };
        // Unchecked: the condition is 0 or 1, so the product cannot exceed `value`.
        self.dfg
            .insert_instruction_and_results(
                Instruction::binary(BinaryOp::Mul { unchecked: true }, value, condition),
                block,
                None,
                call_stack,
            )
            .first()
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    /// The guarded assertion moves into the entry block, multiplied by the condition, and the
    /// diamond becomes straight-line code.
    #[test]
    fn predicates_a_guarded_assertion() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            v2 = array_get v0, index u32 0 -> Field
            constrain v2 == Field 0, \"nonzero\"
            jmp b2()
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.flatten_trivial_conditionals();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            v3 = array_get v0, index u32 0 -> Field
            v4 = cast v1 as Field
            v5 = mul v3, v4
            constrain v5 == Field 0, "nonzero"
            jmp b1()
          b1():
            return
        }
        "#);
    }

    /// Several assertions in one block share a single cast of the condition.
    #[test]
    fn shares_one_cast_between_assertions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            v2 = array_get v0, index u32 0 -> Field
            v3 = array_get v0, index u32 1 -> Field
            constrain v2 == Field 7, \"a\"
            constrain v3 == Field 9, \"b\"
            jmp b2()
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.flatten_trivial_conditionals();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            v3 = array_get v0, index u32 0 -> Field
            v5 = array_get v0, index u32 1 -> Field
            v6 = cast v1 as Field
            v7 = mul v3, v6
            v9 = mul Field 7, v6
            constrain v7 == v9, "a"
            v10 = mul v5, v6
            v12 = mul Field 9, v6
            constrain v10 == v12, "b"
            jmp b1()
          b1():
            return
        }
        "#);
    }

    /// An `array_get` whose index is not provably in bounds can trap, so it has to stay under the
    /// branch for flattening to nullify.
    #[test]
    fn leaves_a_possibly_trapping_read_alone() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1, v2: u32):
            jmpif v1 then: b1(), else: b2()
          b1():
            v3 = array_get v0, index v2 -> Field
            constrain v3 == Field 0, \"nonzero\"
            jmp b2()
          b2():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::flatten_trivial_conditionals);
    }

    /// A store is a side effect flattening has to merge, not something to hoist.
    #[test]
    fn leaves_a_store_alone() {
        let src = "
        acir(inline) fn main f0 {
          b0(v1: u1):
            v2 = allocate -> &mut Field
            store Field 0 at v2
            jmpif v1 then: b1(), else: b2()
          b1():
            store Field 1 at v2
            jmp b2()
          b2():
            v3 = load v2 -> Field
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::flatten_trivial_conditionals);
    }

    /// A conditional with a real else branch is not a diamond this pass can collapse.
    #[test]
    fn leaves_an_if_else_alone() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            v2 = array_get v0, index u32 0 -> Field
            constrain v2 == Field 0, \"then\"
            jmp b3()
          b2():
            v3 = array_get v0, index u32 1 -> Field
            constrain v3 == Field 0, \"else\"
            jmp b3()
          b3():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::flatten_trivial_conditionals);
    }

    /// Brillig branches are executed, so there is no predicate to fold in.
    #[test]
    fn leaves_brillig_alone() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 4], v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            v2 = array_get v0, index u32 0 -> Field
            constrain v2 == Field 0, \"nonzero\"
            jmp b2()
          b2():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::flatten_trivial_conditionals);
    }
}
