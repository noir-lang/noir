//! This module defines a SSA pass that simplifies calls to *pass-through* functions.
//!
//! A pass-through function is one whose every returned value is one of its own parameters
//! ([`crate::ssa::ir::function::Function::pass_through_indices`]) and whose body has no observable side effects
//! ([`Purity::Pure`] after call-graph propagation). Calling such a function produces exactly its
//! arguments and does nothing else, so its call can be replaced by forwarding the
//! matching argument to each result and dropping the call.
//!
//! ## Purity
//!
//! The function must be pure, but Brillig functions are not pure (they are `WithPredicate`)
//! due to side-effect if called from ACIR under a false predicate.
//! However, this pass is only useful for Brillig functions because ACIR functions are
//! always inlined.
//! This pass performs the purity analysis and computes the `WithPredicate` due to ACIR
//! callers, based on the calling context. In any of these cases,
//! we can trust the purity from the custom purity analysis:
//!
//! - the caller is Brillig — Brillig calls are never predicate-masked;
//! - the callee is ACIR — ACIR calls do not zero their outputs
//! - the caller is ACIR, the callee is Brillig, and the call site's predicate is the constant `1`.
//!
//! This pass should be run after flattening but it is still sound otherwise.
//! The pass does not look at the `InlineNever` attribute, so it can remove calls to `InlineNever` functions.

use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{function::FunctionId, instruction::Instruction, value::Value},
    opt::pure::{Purity, compute_function_purities},
    ssa_gen::Ssa,
};

/// A side-effect-free pass-through callee.
struct PassThrough {
    /// For each returned value, the index of the parameter it forwards.
    forwarding: Vec<usize>,
    /// Whether the callee is an ACIR function
    callee_is_acir: bool,
}

impl Ssa {
    /// See the [`passthrough_calls`][self] module for more information.
    pub(crate) fn simplify_passthrough_calls(mut self) -> Ssa {
        // Run the purity analysis and keep only the `Pure` pass-through functions.
        let pass_through: HashMap<FunctionId, PassThrough> = compute_function_purities(&self)
            .intrinsic_purities()
            .filter(|(_, purity)| **purity == Purity::Pure)
            .filter_map(|(id, _)| {
                let function = &self.functions[id];
                let forwarding = function.pass_through_indices()?;
                let callee_is_acir = function.runtime().is_acir();
                Some((*id, PassThrough { forwarding, callee_is_acir }))
            })
            .collect();

        if pass_through.is_empty() {
            return self;
        }

        for function in self.functions.values_mut() {
            let caller_is_brillig = function.runtime().is_brillig();
            // With a flatten CFG, we can rely on enable_side_effects
            let single_block = function.reachable_blocks().len() == 1;

            function.simple_optimization(|context| {
                let (callee_id, arguments) = match context.instruction() {
                    Instruction::Call { func, arguments } => match context.dfg[*func] {
                        Value::Function(callee_id) => (callee_id, arguments.clone()),
                        _ => return,
                    },
                    _ => return,
                };

                let Some(passthrough) = pass_through.get(&callee_id) else {
                    return;
                };

                // A zero predicate can zero brillig outputs, but enable_side_effects is only
                // meaning full on a flatten CFG. If the pass is called pre-flattening, it
                // will still be valid.
                let predicate_is_one =
                    single_block && context.dfg.is_constant_true(context.enable_side_effects);
                let may_forward =
                    caller_is_brillig || passthrough.callee_is_acir || predicate_is_one;
                if !may_forward {
                    return;
                }

                let results = context.dfg.instruction_results(context.instruction_id).to_vec();
                for (result, &parameter_index) in results.iter().zip_eq(&passthrough.forwarding) {
                    context.replace_value(*result, arguments[parameter_index]);
                }
                context.remove_current_instruction();
            });
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    /// Helper function to run the pass for the tests:
    /// We do a `simplify_passthrough_calls` followed by a `remove_unreachable_functions` to remove the simplified callees.
    fn pass_through_and_die(ssa: Ssa) -> Ssa {
        ssa.simplify_passthrough_calls().remove_unreachable_functions()
    }

    #[test]
    fn forwards_acir_passthrough_and_removes_call() {
        // Asserts the un-pruned output: the pass removes the call but leaves the now-dead callee
        // in place (pruning it is a separate pass). The other tests chain `simplify_and_prune`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = call f1(v0, v1) -> u32
            return v3
        }
        acir(fold) fn swap_first f1 {
          b0(v0: u32, v1: u32):
            return v1
        }
        ";
        let ssa = pass_through_and_die(Ssa::from_str(src).unwrap());
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            return v1
        }
        acir(fold) fn swap_first f1 {
          b0(v0: u32, v1: u32):
            return v1
        }
        ");
    }

    #[test]
    fn forwards_permuted_returns() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3, v4 = call f1(v0, v1) -> (u32, u32)
            v5 = add v3, v4
            return v5
        }
        acir(fold) fn swap f1 {
          b0(v0: u32, v1: u32):
            return v1, v0
        }
        ";
        // `f1` is `acir(fold)` so it is an entry point, `remove_unreachable_functions()` won't work for it.
        let ssa = pass_through_and_die(Ssa::from_str(src).unwrap());
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = add v1, v0
            return v2
        }
        acir(fold) fn swap f1 {
          b0(v0: u32, v1: u32):
            return v1, v0
        }
        ");
    }

    #[test]
    fn forwards_brillig_passthrough_called_from_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            return v2
        }
        brillig(inline_never) fn id f1 {
          b0(v0: u32):
            return v0
        }
        ";
        let ssa = pass_through_and_die(Ssa::from_str(src).unwrap());
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            return v0
        }
        ");
    }

    #[test]
    fn does_not_forward_brillig_passthrough_called_from_acir_under_unknown_predicate() {
        // ACIR→brillig: the brillig outputs are zeroed under a disabled predicate, so forwarding
        // the argument would be wrong unless the predicate is known to be one.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            enable_side_effects v1
            v3 = call f1(v0) -> u32
            return v3
        }
        brillig(inline_never) fn id f1 {
          b0(v0: u32):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, pass_through_and_die);
    }

    #[test]
    fn does_not_forward_constraining_function() {
        // `f1` forwards `v1` but also constrains `v0`; dropping the call would drop the constraint.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = call f1(v0, v1) -> u32
            return v3
        }
        acir(inline) fn checked f1 {
          b0(v0: u32, v1: u32):
            constrain v0 == u32 0
            return v1
        }
        ";
        assert_ssa_does_not_change(src, pass_through_and_die);
    }

    #[test]
    fn forwards_acir_to_brillig_passthrough_under_known_predicate() {
        // The caller is a single block with no `enable_side_effects`, so the predicate is the
        // constant one: the brillig outputs cannot be zeroed and the argument may be forwarded.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            return v2
        }
        brillig(inline_never) fn id f1 {
          b0(v0: u32):
            return v0
        }
        ";
        let ssa = pass_through_and_die(Ssa::from_str(src).unwrap());
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            return v0
        }
        ");
    }

    #[test]
    fn does_not_forward_acir_to_brillig_from_a_multi_block_caller() {
        // The caller is not flattened (more than one block), so `enable_side_effects` does not
        // capture control-flow conditionality and the predicate cannot be trusted — even though
        // the call sits in the unconditionally-executed entry block.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            v3 = call f1(v0) -> u32
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b2()
          b2():
            return v3
        }
        brillig(inline_never) fn id f1 {
          b0(v0: u32):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, pass_through_and_die);
    }

    #[test]
    fn does_not_forward_passthrough_that_transitively_constrains() {
        // `f1` structurally returns its input and is locally side-effect-free, but it calls `f2`
        // which constrains. Call-graph propagation makes `f1` `PureWithPredicate`, so dropping a
        // call to `f1` would drop `f2`'s constraint.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            return v2
        }
        acir(fold) fn forwards_but_calls_constrain f1 {
          b0(v0: u32):
            v2 = call f2(v0) -> u32
            return v0
        }
        acir(fold) fn constrains f2 {
          b0(v0: u32):
            constrain v0 == u32 0
            return v0
        }
        ";
        assert_ssa_does_not_change(src, pass_through_and_die);
    }

    #[test]
    fn removes_call_to_side_effect_free_void_passthrough() {
        // A pure function that returns nothing forwards zero values, so its call is simply dropped.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            call f1(v0)
            return v0
        }
        acir(fold) fn does_nothing f1 {
          b0(v0: u32):
            return
        }
        ";
        // The callee is `acir(fold)`, i.e. its own ACIR entry point, so it survives
        // `remove_unreachable_functions` and there is nothing to prune.
        let ssa = pass_through_and_die(Ssa::from_str(src).unwrap());
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            return v0
        }
        acir(fold) fn does_nothing f1 {
          b0(v0: u32):
            return
        }
        ");
    }
}
