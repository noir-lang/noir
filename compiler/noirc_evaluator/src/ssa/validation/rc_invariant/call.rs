//! The call-argument reachable-aliased-use check.
//!
//! A callee may mutate an array argument in place at runtime — directly, or by
//! returning an alias of it that the caller then mutates ([`super::Context`]'s
//! alias analysis treats `Call` results as fresh, so that escape is invisible
//! to the `array_set` scan). When the caller reuses such an argument without a
//! protecting `inc_rc`, the mutation becomes observable through the alias —
//! exactly the precondition `purity_analysis` relies on being absent.
//!
//! This verifier drives the same aliasing/coverage/forward-walk machinery as
//! [`super::array_set`], seeded from call arguments instead of `array_set`
//! sources, and gated on whether the callee can modify its arguments (mirroring
//! `can_modify_args` in `ssa_gen`).

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            function::{Function, FunctionId},
            instruction::{Instruction, Intrinsic, TerminatorInstruction},
            value::{Value, ValueId},
        },
        opt::pure::Purity,
        ssa_gen::Ssa,
    },
};

use super::Context;

/// Verify the call-argument aliasing invariant on every Brillig function in
/// `ssa`. See the module docs for the invariant.
pub(crate) fn verify(ssa: &Ssa) -> RtResult<()> {
    let may_mutate = compute_may_mutate_args(ssa);
    let returns_arg_alias = compute_returns_arg_alias(ssa);

    // A user-function call must have its array arguments checked if the callee
    // may mutate one in place, *or* may hand back an alias of one that the
    // caller then mutates (e.g. an identity function — see [#1443]). Either way,
    // reusing the argument without a protecting `inc_rc` is a hazard. Both
    // summaries cover every function, so we can index them directly.
    //
    // [#1443]: https://github.com/noir-lang/noir-claude/issues/1443
    let needs_check: HashMap<FunctionId, bool> =
        ssa.functions.keys().map(|id| (*id, may_mutate[id] || returns_arg_alias[id])).collect();

    for function in ssa.functions.values() {
        verify_function(function, &needs_check)?;
    }
    Ok(())
}

/// Per-function check. Skips ACIR functions (the invariant only applies to
/// Brillig, where a callee may mutate an argument in place). For every `call`
/// whose callee may mutate an argument or return an alias of one (`needs_check`),
/// treats each array-typed argument as an all-index in-place mutation and runs
/// the shared coverage + forward walk: a forward-reachable aliased read with no
/// protecting `inc_rc` is a hazard.
fn verify_function(function: &Function, needs_check: &HashMap<FunctionId, bool>) -> RtResult<()> {
    if !function.runtime().is_brillig() {
        return Ok(());
    }

    let ctx = Context::new(function);

    for block_id in function.reachable_blocks() {
        for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            let instruction_id = *instruction_id;
            let Instruction::Call { func, arguments } = &function.dfg[instruction_id] else {
                continue;
            };

            // Mirror `ssa_gen`'s `can_modify_args`: a callee that provably
            // cannot mutate an argument *or* hand back an alias of one (a
            // foreign call, a pure builtin, or a function that neither mutates
            // an argument nor returns an alias of one) has its argument clones
            // elided by the ownership pass, so a reused argument legitimately
            // carries no `inc_rc`. Skipping such calls is what keeps this check
            // from flagging well-formed SSA.
            if !callee_needs_arg_check(function, *func, needs_check) {
                continue;
            }

            // Treat the call as an all-index mutation of each array-*value*
            // argument: `derived` is empty (the callee has no in-place result
            // to chain from here) and the write index is unknown (it may touch
            // any position). `is_array` matches a top-level array/vector value
            // and so excludes a reference argument (`&mut [T; N]`): mutation
            // through a reference is the explicit, caller-visible pattern the
            // frontend passes a `&mut` for, not the value-array copy-on-write
            // hazard this check is about (and a reference param already makes
            // the callee impure). A top-level array of references (`[&mut T;
            // N]`) is still a value and is checked.
            let array_args: Vec<ValueId> = arguments
                .iter()
                .copied()
                .filter(|&arg| function.dfg.type_of_value(arg).is_array())
                .collect();
            for arg in array_args {
                let Some(hit) = ctx.aliased_use_for_source(
                    arg,
                    block_id,
                    idx,
                    instruction_id,
                    None,
                    im::HashSet::new(),
                ) else {
                    continue;
                };

                let message = format!(
                    "call in function {} passes array {arg} that is read again as {} on a \
                     forward path with no preceding `inc_rc`; if the callee mutates the argument \
                     in place, or returns an alias of it that is then mutated, the mutation would \
                     be observable through that alias",
                    function.name(),
                    hit.value,
                );
                return Err(RuntimeError::CallArgAliasViolation {
                    message,
                    call_stack: function.dfg.get_instruction_call_stack(instruction_id),
                    aliased_use_call_stack: function
                        .dfg
                        .get_instruction_call_stack(hit.instruction),
                });
            }
        }
    }
    Ok(())
}

/// Whether a call to the callee referenced by `func` needs its array arguments
/// checked — i.e. the callee may mutate an argument in place or return an alias
/// of one. Mirrors `ssa_gen`'s `can_modify_args`: foreign calls only read their
/// inputs and return fresh results; pure builtins that are safe for clone
/// elision do neither; an unresolved/dynamic callee is assumed to need
/// checking; and a known function is looked up in the combined call-graph
/// summary (`may_mutate || returns_arg_alias`).
fn callee_needs_arg_check(
    function: &Function,
    func: ValueId,
    needs_check: &HashMap<FunctionId, bool>,
) -> bool {
    match &function.dfg[func] {
        Value::Function(callee) => needs_check.get(callee).copied().unwrap_or(true),
        Value::Intrinsic(intrinsic) => intrinsic_may_mutate_args(*intrinsic),
        Value::ForeignFunction { .. } => false,
        _ => true,
    }
}

/// Whether a call to `intrinsic` may mutate an array argument in place,
/// mirroring `is_pure_builtin_func` in `ssa_gen`: a pure intrinsic that is safe
/// for clone elision in Brillig cannot, everything else conservatively can.
fn intrinsic_may_mutate_args(intrinsic: Intrinsic) -> bool {
    intrinsic.unsafe_for_clone_elision_in_brillig()
        || !matches!(intrinsic.purity(), Purity::Pure | Purity::PureWithPredicate)
}

/// Compute, for every function, whether a call to it may mutate the storage of
/// one of its array arguments observably to the caller.
///
/// A function may-mutate if it contains an in-place mutation (`array_set` or
/// `store`), calls a may-mutate function, calls a mutating intrinsic, or calls
/// an unresolved/dynamic target (assume the worst). Foreign calls contribute
/// nothing — oracles only read their inputs. This is an over-approximation:
/// the only callees marked *not* may-mutate are exactly those whose argument
/// clones the ownership pass elides, so a reused argument with no `inc_rc`
/// never trips the check on well-formed SSA. Propagated to a fixed point over
/// the call graph.
fn compute_may_mutate_args(ssa: &Ssa) -> HashMap<FunctionId, bool> {
    let mut may_mutate: HashMap<FunctionId, bool> = HashMap::default();
    let mut callees: HashMap<FunctionId, Vec<FunctionId>> = HashMap::default();

    for function in ssa.functions.values() {
        let mut base = false;
        let mut calls = Vec::new();
        for block_id in function.reachable_blocks() {
            for instruction_id in function.dfg[block_id].instructions() {
                match &function.dfg[*instruction_id] {
                    Instruction::ArraySet { .. } | Instruction::Store { .. } => base = true,
                    Instruction::Call { func, .. } => match &function.dfg[*func] {
                        Value::Function(callee) => calls.push(*callee),
                        Value::Intrinsic(intrinsic) => {
                            base |= intrinsic_may_mutate_args(*intrinsic);
                        }
                        // Foreign calls only read their inputs.
                        Value::ForeignFunction { .. } => {}
                        // An unresolved or dynamic callee: assume the worst.
                        _ => base = true,
                    },
                    _ => {}
                }
            }
        }
        may_mutate.insert(function.id(), base);
        callees.insert(function.id(), calls);
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (&id, callee_ids) in &callees {
            if may_mutate[&id] {
                continue;
            }
            if callee_ids.iter().any(|c| may_mutate.get(c).copied().unwrap_or(true)) {
                may_mutate.insert(id, true);
                changed = true;
            }
        }
    }

    may_mutate
}

/// Compute, for every function, whether it may return an array value that
/// aliases one of its array parameters — e.g. an identity function that returns
/// its input unchanged. Such a call hands the caller an alias of the argument,
/// so an in-place mutation of the *result* mutates the *argument's* storage; the
/// call must be checked even when the callee does not itself mutate.
///
/// Distinct from "returns any array": a callee that returns a *fresh* array (a
/// `make_array`, or a foreign/intrinsic-call result — the shape of an oracle
/// wrapper) is not flagged, so its caller's clone-elided arguments stay
/// accepted. Propagated to a fixed point over the call graph because the
/// alias property flows through `Value::Function` call results.
fn compute_returns_arg_alias(ssa: &Ssa) -> HashMap<FunctionId, bool> {
    let mut returns_arg_alias: HashMap<FunctionId, bool> =
        ssa.functions.keys().map(|id| (*id, false)).collect();

    // Monotonic fixed point: a function only ever flips from false to true, and
    // `function_returns_arg_alias` reads the current map for callee results.
    let mut changed = true;
    while changed {
        changed = false;
        for function in ssa.functions.values() {
            if returns_arg_alias[&function.id()] {
                continue;
            }
            if function_returns_arg_alias(function, &returns_arg_alias) {
                returns_arg_alias.insert(function.id(), true);
                changed = true;
            }
        }
    }

    returns_arg_alias
}

/// Whether `function` returns an array value that may alias one of its array
/// parameters, given the current `returns_arg_alias` summary for resolving
/// callee results.
///
/// Computes the set of *parameter-derived* values to a fixed point: an array
/// parameter, a block parameter threaded from one, an `array_set` of one, or a
/// `Value::Function` call result whose callee `returns_arg_alias` and is fed a
/// parameter-derived argument. `make_array`, foreign-call and intrinsic results
/// are fresh and stop the trace. The function returns an arg alias iff any
/// returned value is parameter-derived.
fn function_returns_arg_alias(
    function: &Function,
    returns_arg_alias: &HashMap<FunctionId, bool>,
) -> bool {
    let dfg = &function.dfg;

    // Incoming block-parameter arguments per destination block, to thread
    // parameter-derived-ness across edges.
    let mut incoming: HashMap<BasicBlockId, Vec<Vec<ValueId>>> = HashMap::default();
    for block_id in function.reachable_blocks() {
        match dfg[block_id].terminator() {
            Some(TerminatorInstruction::Jmp { destination, arguments, .. }) => {
                incoming.entry(*destination).or_default().push(arguments.clone());
            }
            Some(TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            }) => {
                incoming.entry(*then_destination).or_default().push(then_arguments.clone());
                incoming.entry(*else_destination).or_default().push(else_arguments.clone());
            }
            _ => {}
        }
    }

    // Seed with the function's array-value parameters (entry block parameters).
    let entry = function.entry_block();
    let mut param_derived: HashSet<ValueId> = dfg
        .block_parameters(entry)
        .iter()
        .copied()
        .filter(|&p| dfg.type_of_value(p).is_array())
        .collect();

    let mut changed = true;
    while changed {
        changed = false;

        for block_id in function.reachable_blocks() {
            // Block parameters fed a parameter-derived argument on some edge.
            if let Some(edges) = incoming.get(&block_id) {
                let params = dfg.block_parameters(block_id);
                for (i, &param) in params.iter().enumerate() {
                    if param_derived.contains(&param) {
                        continue;
                    }
                    // Check any of the incoming edges for arguments which are derived from function inputs.
                    let fed = edges
                        .iter()
                        .any(|args| args.get(i).is_some_and(|a| param_derived.contains(a)));
                    if fed {
                        param_derived.insert(param);
                        changed = true;
                    }
                }
            }

            for instruction_id in dfg[block_id].instructions() {
                let propagate = match &dfg[*instruction_id] {
                    // An array_set result shares the operand's storage.
                    Instruction::ArraySet { array, .. } => param_derived.contains(array),
                    // A call result aliases an argument only if the callee
                    // returns an arg alias and is fed a parameter-derived
                    // argument. Foreign/intrinsic results are fresh.
                    Instruction::Call { func, arguments } => match &dfg[*func] {
                        Value::Function(callee) => {
                            returns_arg_alias[callee]
                                && arguments.iter().any(|a| param_derived.contains(a))
                        }
                        _ => false,
                    },
                    _ => false,
                };
                if propagate {
                    for &result in dfg.instruction_results(*instruction_id) {
                        if dfg.type_of_value(result).is_array() && param_derived.insert(result) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    // Check if the return block contains a value that was derived from the inputs.
    function.reachable_blocks().iter().any(|&block_id| {
        matches!(
            dfg[block_id].terminator(),
            Some(TerminatorInstruction::Return { return_values, .. })
                if return_values.iter().any(|v| param_derived.contains(v))
        )
    })
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_verifier_accepts_because;
    use crate::ssa::ssa_gen::Ssa;

    /// Parse `src`, run the `call` verifier, and require it to reject the SSA
    /// with a [`crate::errors::RuntimeError::CallArgAliasViolation`]. Panics on
    /// any other outcome. Runs `call::verify` directly (not the combined check)
    /// so the assertion proves the *call* verifier is the one that caught the
    /// hazard.
    fn assert_verifier_rejects(src: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        let err = super::verify(&ssa).expect_err("expected the verifier to reject");
        assert!(
            matches!(err, crate::errors::RuntimeError::CallArgAliasViolation { .. }),
            "expected CallArgAliasViolation, got {err:?}",
        );
    }

    /// Regression for noir-lang/noir-claude#1426. The ownership pass clones
    /// (`inc_rc`s) every non-last use of an array, so a well-formed program
    /// that reuses an array across a call always RC-protects it. This
    /// hand-written SSA omits those bumps: a pure identity callee (`f2`)
    /// returns the array input unchanged, `f1` then `array_set`s the returned
    /// alias — mutating its caller's array in place at RC 1 — and `main`
    /// reuses the same array across two calls to `f1`. The in-place mutation
    /// is therefore observable to `main` (the first call's mutation is seen by
    /// the second), which is exactly the precondition `purity_analysis`
    /// relies on being absent. The verifier must reject: both the reused arg
    /// in `main` and the reused-then-read arg in `f1` lack a preceding
    /// `inc_rc`.
    #[test]
    fn end_to_end_array_reused_across_call_without_inc_rc_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = call f1(v0) -> Field
                v2 = call f1(v0) -> Field
                return v1, v2
            }
            brillig(inline) fn bump_via_identity f1 {
              b0(v0: [Field; 2]):
                v1 = call f2(v0) -> [Field; 2]
                v2 = array_get v0, index u32 0 -> Field
                v3 = add v2, Field 1
                v4 = array_set v1, index u32 0, value v3
                return v3
            }
            brillig(inline) fn identity f2 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_rejects(src);
    }

    /// The well-formed counterpart of
    /// [`end_to_end_array_reused_across_call_without_inc_rc_is_rejected`]: the
    /// `inc_rc`s the ownership pass emits are present — in `main` before the
    /// reused call arg, and in `f1` before the array escapes to `identity` and
    /// is read again. Every reused array call-arg is now RC-protected, so the
    /// in-place mutation cannot be observed through an alias and the verifier
    /// accepts. This pins down that the call-arg check credits a preceding
    /// `inc_rc` rather than flagging every reused call arg unconditionally.
    #[test]
    fn end_to_end_array_reused_across_call_with_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                inc_rc v0
                v1 = call f1(v0) -> Field
                v2 = call f1(v0) -> Field
                return v1, v2
            }
            brillig(inline) fn bump_via_identity f1 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v1 = call f2(v0) -> [Field; 2]
                v2 = array_get v0, index u32 0 -> Field
                v3 = add v2, Field 1
                v4 = array_set v1, index u32 0, value v3
                return v3
            }
            brillig(inline) fn identity f2 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_accepts_because(
            src,
            "every reused array call-arg is protected by a preceding inc_rc",
        );
    }

    /// Reduced from the `array_sort` execution test (`quicksort`): a `&mut`
    /// **reference** to an array is passed to a callee that sorts it in place
    /// and then loaded back. The argument is a reference, not an array value,
    /// so it is *not* a copy-on-write hazard — mutation through a `&mut` is the
    /// explicit, caller-visible pattern the frontend passes a reference for
    /// (and a reference parameter already makes the callee impure). The call
    /// verifier must skip reference arguments and accept; flagging this was a
    /// false positive fixed by the `contains_reference` filter.
    #[test]
    fn end_to_end_reference_argument_read_back_after_call_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u8; 3]):
                v1 = allocate -> &mut [u8; 3]
                store v0 at v1
                call f1(v1)
                v2 = load v1 -> [u8; 3]
                return v2
            }
            brillig(inline) fn sort_in_place f1 {
              b0(v0: &mut [u8; 3]):
                v1 = load v0 -> [u8; 3]
                v3 = array_set v1, index u32 0, value u8 9
                store v3 at v0
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "the call argument is a &mut reference, not an array value, so it is not a COW hazard",
        );
    }

    /// Regression for noir-lang/noir-claude#1443. `identity` (`f1`) does not
    /// mutate its argument, so `callee_may_mutate_args` is `false`; but by
    /// returning `v0` unchanged it makes the call result `v1` an **alias** of
    /// `v0`. The caller then `array_set v1` (mutating `v0`'s storage in place at
    /// RC 1) and reads `v0` afterwards, observing the mutation. The frontend
    /// would emit an `inc_rc v0` before the call (`v0` is reused), so this SSA
    /// is malformed. The call verifier must not skip a callee that may return an
    /// alias of an array input — `returns_arg_alias` — and so flags the reused
    /// `v0`.
    #[test]
    fn end_to_end_callee_returns_input_alias_mutated_by_caller_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = call f1(v0) -> [Field; 2]
                v2 = array_set v1, index u32 0, value Field 9
                v3 = array_get v0, index u32 0 -> Field
                return v3
            }
            brillig(inline) fn identity f1 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_rejects(src);
    }

    /// The well-formed counterpart of
    /// [`end_to_end_callee_returns_input_alias_mutated_by_caller_is_rejected`]:
    /// the `inc_rc v0` the ownership pass emits before the reused call argument
    /// is present, so the later `array_set` copies rather than mutating `v0` in
    /// place and the read of `v0` is sound. Accepted.
    #[test]
    fn end_to_end_callee_returns_input_alias_with_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                inc_rc v0
                v1 = call f1(v0) -> [Field; 2]
                v2 = array_set v1, index u32 0, value Field 9
                v3 = array_get v0, index u32 0 -> Field
                return v3
            }
            brillig(inline) fn identity f1 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_accepts_because(
            src,
            "the reused argument is protected by a preceding inc_rc",
        );
    }

    /// Soundness guard for `returns_arg_alias`: a callee that returns a *fresh*
    /// array (here a foreign-call result, the shape of an oracle wrapper that
    /// returns an array) does **not** alias its input. Even though the caller
    /// reuses the argument with no `inc_rc` — which the frontend legitimately
    /// elides for oracle wrappers — there is no aliasing hazard, so the call
    /// verifier must skip it and accept. A coarser "returns any array" rule
    /// would have falsely flagged this.
    #[test]
    fn end_to_end_callee_returns_fresh_array_reused_arg_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = call f1(v0) -> [Field; 2]
                v2 = array_set v1, index u32 0, value Field 9
                v3 = array_get v0, index u32 0 -> Field
                return v3
            }
            brillig(inline) fn wrapper f1 {
              b0(v0: [Field; 2]):
                v1 = call my_oracle(v0) -> [Field; 2]
                return v1
            }"#;
        assert_verifier_accepts_because(
            src,
            "the callee returns a fresh foreign-call result, not an alias of its input",
        );
    }
}
