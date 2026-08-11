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
//!
//! It additionally checks a relation only visible at the call site: whether two
//! argument positions of one call denote the same storage with no protecting
//! `inc_rc` (see [`check_co_aliased_arguments`]).

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            function::{Function, FunctionId},
            instruction::{Instruction, InstructionId, Intrinsic, TerminatorInstruction},
            types::Type,
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
    // summaries cover every function, so indexing them is safe (and asserts we
    // computed a value for every callee).
    //
    // [#1443]: https://github.com/noir-lang/noir-claude/issues/1443
    let needs_check = |callee: FunctionId| may_mutate[&callee] || returns_arg_alias[&callee];

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
fn verify_function(function: &Function, needs_check: &impl Fn(FunctionId) -> bool) -> RtResult<()> {
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
                    imbl::HashSet::new(),
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

            check_co_aliased_arguments(function, &ctx, arguments, block_id, idx, instruction_id)?;
        }
    }
    Ok(())
}

/// Reject a `call` in which two argument positions may denote the same array
/// storage with no protecting `inc_rc`.
///
/// The per-argument forward walk in [`verify_function`] looks for an aliased
/// read *in the caller, after the call*, so it cannot see the hazard where the
/// same buffer reaches the callee twice — as the same value in two positions
/// (`f(v, v)`), or once as an array value and once as the pointee of a
/// reference (the shape `ssa_gen` emits for `f(&mut x, x)`). The callee can
/// then mutate one handle in place at reference count 1 and observe (e.g.
/// return) the pre-mutation contents through the other, entirely inside the
/// callee — where the two parameters are unrelated values and the shared-buffer
/// relation is invisible to the intraprocedural alias engine. The relation is
/// only established here, at the call site, so this is where it must be checked
/// (noir-lang/noir-claude#1563).
///
/// Each argument position is resolved to the storage it denotes at the call:
/// an array-typed argument to itself, and a reference argument to the value
/// its in-block reaching `store` wrote, when there is one (after
/// `mem2reg_brillig` the emitted shape keeps that `store` in the call's own
/// block; a pointee established elsewhere is not traced — a false negative,
/// never a false positive). Two *reference* arguments sharing a pointee are
/// not flagged: a write through a reference is ordinary reference semantics,
/// visible through every alias of the reference by design, not a copy-on-write
/// violation — so a pair must include at least one array-value position.
///
/// Whether a pair denotes one buffer, and whether that buffer is protected, is
/// decided per path by [`Context::pair_has_unprotected_shared_storage`]: a
/// pair is flagged only when some backward path resolves both positions to the
/// same storage with no `inc_rc` crossed on it. A whole-call relation is not
/// path-sensitive enough — a branch-local `inc_rc` protecting the only sharing
/// path, or two branches passing one buffer through *different* positions,
/// must both be accepted.
fn check_co_aliased_arguments(
    function: &Function,
    ctx: &Context,
    arguments: &[ValueId],
    block_id: BasicBlockId,
    call_idx: usize,
    call_id: InstructionId,
) -> RtResult<()> {
    // For each argument position: `is_value` (an array value, as opposed to a
    // reference whose pointee was resolved) and the storage the position
    // denotes. `None` when the position denotes no array storage we can
    // resolve.
    let storages: Vec<Option<(bool, ValueId)>> = arguments
        .iter()
        .map(|&arg| match function.dfg.type_of_value(arg).as_ref() {
            typ if typ.is_array() => Some((true, arg)),
            Type::Reference(element, _) if element.contains_an_array() => {
                let pointee = in_block_reaching_store(function, block_id, call_idx, arg)?;
                Some((false, pointee))
            }
            _ => None,
        })
        .collect();

    for (i, i_storage) in storages.iter().enumerate() {
        let Some((i_is_value, i_value)) = i_storage else { continue };
        for (j, j_storage) in storages.iter().enumerate().skip(i + 1) {
            let Some((j_is_value, j_value)) = j_storage else { continue };
            if !i_is_value && !j_is_value {
                continue;
            }
            if !ctx.pair_has_unprotected_shared_storage(*i_value, *j_value, block_id, call_idx) {
                continue;
            }
            let message = format!(
                "call in function {} passes the same array storage through two arguments \
                 ({} and {}) with no preceding `inc_rc`; if the callee mutates it in place \
                 through one of them, the mutation would be observable through the other",
                function.name(),
                arguments[i],
                arguments[j],
            );
            let call_stack = function.dfg.get_instruction_call_stack(call_id);
            return Err(RuntimeError::CallArgAliasViolation {
                message,
                call_stack: call_stack.clone(),
                aliased_use_call_stack: call_stack,
            });
        }
    }
    Ok(())
}

/// The value most recently stored through `address` in `block_id` before the
/// instruction at `call_idx` — the reference's in-block reaching definition —
/// or `None` when the block contains no such store.
fn in_block_reaching_store(
    function: &Function,
    block_id: BasicBlockId,
    call_idx: usize,
    address: ValueId,
) -> Option<ValueId> {
    let instructions = function.dfg[block_id].instructions();
    instructions[..call_idx].iter().rev().find_map(|id| match &function.dfg[*id] {
        Instruction::Store { address: a, value } if *a == address => Some(*value),
        _ => None,
    })
}

/// Whether a call to the callee referenced by `func` needs its array arguments
/// checked — i.e. the callee may mutate an argument in place or return an alias
/// of one. Mirrors `ssa_gen`'s `can_modify_args`: foreign calls only read their
/// inputs and return fresh results; pure builtins that are safe for clone
/// elision do neither; an unresolved/dynamic callee is assumed to need
/// checking; and a known function is decided by `needs_check`, which combines
/// the two call-graph summaries (`may_mutate || returns_arg_alias`).
fn callee_needs_arg_check(
    function: &Function,
    func: ValueId,
    needs_check: &impl Fn(FunctionId) -> bool,
) -> bool {
    match &function.dfg[func] {
        Value::Function(callee) => needs_check(*callee),
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

/// Populate per-function state with `init`, then run `update` over every
/// function until a full round makes no change, and return the final state.
///
/// `init` receives each function and a mutable reference to the
/// default-initialized state to fill in that function's entry. `update`
/// likewise receives each function and the state, mutates it in place, and
/// returns whether it changed anything; returning `false` short-circuits a
/// function whose contribution is already settled. The caller keeps whichever
/// part of the final state it needs.
fn fixpoint<S: Default>(
    ssa: &Ssa,
    mut init: impl FnMut(&Function, &mut S),
    mut update: impl FnMut(&Function, &mut S) -> bool,
) -> S {
    let mut state = S::default();
    for function in ssa.functions.values() {
        init(function, &mut state);
    }
    let mut changed = true;
    while changed {
        changed = false;
        for function in ssa.functions.values() {
            changed |= update(function, &mut state);
        }
    }
    state
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
    let (may_mutate, _callees) = fixpoint(
        ssa,
        // Initialize the state to each function's own (non-propagated) may-mutate flag,
        // plus its callee list. The callee list, built once by `init`, lets the propagation
        // rounds avoid re-scanning instruction bodies.
        |function,
         (may_mutate, callees): &mut (
            HashMap<FunctionId, bool>,
            HashMap<FunctionId, Vec<FunctionId>>,
        )| {
            let mut base = false;
            let mut calls = Vec::new();
            for block_id in function.reachable_blocks() {
                for instruction_id in function.dfg[block_id].instructions() {
                    match &function.dfg[*instruction_id] {
                        Instruction::ArraySet { .. } => base = true,
                        // A `store` can only mutate an array argument's storage
                        // when the stored value is (or contains) an array — e.g.
                        // writing a new array through a `&mut [T]` parameter. A
                        // store of a purely numeric value (such as a wrapper
                        // materializing a scalar `&mut` argument) cannot touch any
                        // array argument, so counting it would mark input-preserving
                        // wrappers as may-mutate and reject SSA whose call-argument
                        // clones the ownership pass legitimately elided.
                        Instruction::Store { value, .. } => {
                            base |= function.dfg.type_of_value(*value).contains_an_array();
                        }
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
        },
        // Propagate: a function may-mutate if any callee may-mutate.
        |function, (may_mutate, callees)| {
            let id = function.id();
            if may_mutate[&id] {
                return false;
            }
            let now = callees[&id].iter().any(|c| may_mutate[c]);
            if now {
                may_mutate.insert(id, true);
            }
            now
        },
    );
    may_mutate
}

/// Compute, for every function, whether it may return an array value that
/// aliases one of its array parameters — e.g. an identity function that returns
/// its input unchanged. Such a call hands the caller an alias of the argument,
/// so an in-place mutation of the *result* mutates the *argument's* storage; the
/// call must be checked even when the callee does not itself mutate.
///
/// Distinct from "returns any array": a callee that returns a *fresh* array (a
/// `make_array`, or a foreign-call result — the shape of an oracle wrapper that
/// returns an array) is not flagged, so its caller's clone-elided arguments stay
/// accepted. Propagated to a fixed point over the call graph because the alias
/// property flows through `Value::Function` call results. (Alias-returning
/// intrinsics such as the vector mutators are left to `may_mutate` — see
/// [`function_returns_arg_alias`].)
fn compute_returns_arg_alias(ssa: &Ssa) -> HashMap<FunctionId, bool> {
    // Monotonic fixed point: every function starts `false`; one only ever flips
    // to true, and `function_returns_arg_alias` reads the current map to resolve
    // callee results.
    fixpoint(
        ssa,
        |function, returns_arg_alias: &mut HashMap<FunctionId, bool>| {
            returns_arg_alias.insert(function.id(), false);
        },
        |function, returns_arg_alias| {
            let id = function.id();
            if returns_arg_alias[&id] {
                return false;
            }
            let now = function_returns_arg_alias(function, returns_arg_alias);
            if now {
                returns_arg_alias.insert(id, true);
            }
            now
        },
    )
}

/// Whether `function` returns an array value that may alias one of its array
/// parameters, given the current `returns_arg_alias` summary for resolving
/// callee results.
///
/// Computes the set of *parameter-derived* values to a fixed point: an array
/// parameter, a block parameter threaded from one, an `array_set` or nested
/// `array_get` of one, or a `Value::Function` call result whose callee
/// `returns_arg_alias` and is fed a parameter-derived argument. `make_array` and
/// foreign-call results stop the trace (genuinely fresh). Intrinsic results also
/// stop the trace here — the alias-returning ones
/// (`unsafe_for_clone_elision_in_brillig`) are instead covered by `may_mutate`,
/// since calling one sets that flag. The function returns an arg alias iff any
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
                    // A *nested* array_get returns a sub-array that shares the
                    // source's storage (a brillig array_get on a nested array
                    // aliases rather than copies). The `is_array` gate on the
                    // result below restricts this to the nested case — a
                    // non-nested get yields a scalar and propagates nothing.
                    Instruction::ArrayGet { array, .. } => param_derived.contains(array),
                    // A user-function call result aliases an argument only if
                    // the callee returns an arg alias and is fed a
                    // parameter-derived argument.
                    //
                    // Foreign-call results are genuinely fresh (oracles copy
                    // across the boundary). Intrinsic results are *not* always
                    // fresh — the `unsafe_for_clone_elision_in_brillig`
                    // intrinsics (vector mutators, `str_as_bytes`,
                    // `array_as_str_unchecked`) return an alias of their input —
                    // but we deliberately don't trace through them here: calling
                    // one already makes the function `may_mutate`
                    // (`intrinsic_may_mutate_args`), so `needs_check` flags it
                    // via that summary. This pass only has to cover the gap
                    // `may_mutate` misses: a non-mutating function that passes an
                    // input straight back.
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

    /// `returns_arg_alias` must trace through a **nested** `array_get`: a brillig
    /// `array_get` on a nested array returns a sub-array that *aliases* the
    /// source's storage (the same "the input was moved" case `pure.rs` models).
    /// Here `f1` returns `v0[0]`, an alias of its nested-array input; the caller
    /// `array_set`s that result (mutating the input's storage in place) and then
    /// reads the input. With no `inc_rc` on the reused argument the verifier must
    /// reject.
    #[test]
    fn end_to_end_callee_returns_nested_array_get_alias_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [[u8; 3]; 2]):
                v1 = call f1(v0) -> [u8; 3]
                v2 = array_set v1, index u32 0, value u8 9
                v3 = array_get v0, index u32 0 -> [u8; 3]
                return v3
            }
            brillig(inline) fn nested_identity f1 {
              b0(v0: [[u8; 3]; 2]):
                v1 = array_get v0, index u32 0 -> [u8; 3]
                return v1
            }"#;
        assert_verifier_rejects(src);
    }

    /// Regression for the `acir_vs_brillig` AST-fuzzer seed `0x3f11548500100000`.
    /// `wrapper` (`f1`) is an input-preserving forwarder — it only reads its
    /// array argument (the real program `println`s it) and its single `store`
    /// merely materializes a scalar `&mut` argument into a fresh local
    /// allocation. That scalar store cannot mutate any array argument's storage,
    /// so the ownership pass classifies `f1` as unable to modify its arguments
    /// (`is_oracle_wrapper`) and elides the clone the caller would otherwise emit
    /// before the reused array arg. `main` accordingly passes `v0` with no
    /// preceding `inc_rc` and reads it back after the call. The call verifier
    /// must agree: gating a `store` on whether the stored value contains an array
    /// keeps `f1` off `may_mutate`, so the reused argument is not flagged. Before
    /// that gate the scalar store marked `f1` may-mutate and this well-formed SSA
    /// was rejected — an internal-compiler-error on valid code in debug builds.
    #[test]
    fn end_to_end_callee_with_scalar_store_only_reused_arg_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [] : [i16]
                call f1(v0, u32 25)
                inc_rc v0
                v3 = array_get v0, index u32 0 -> i16
                return v3
            }
            brillig(inline) fn wrapper f1 {
              b0(v0: [i16], v1: u32):
                v2 = allocate -> &mut u32
                store v1 at v2
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "a scalar store cannot mutate an array argument, so the reused arg is not a hazard",
        );
    }

    /// Regression for noir-lang/noir-claude#1563. The same buffer reaches the
    /// callee through **two argument positions of one call**: `v1` is a `&mut`
    /// reference whose pointee (the dominating `store v0 at v1`) is also passed
    /// by value as the second argument, with no `inc_rc` protecting it. The
    /// callee mutates the buffer in place through the reference (RC is 1) and
    /// returns its by-value parameter, so the pre-mutation snapshot the
    /// by-value argument is supposed to be is observably corrupted — running
    /// this SSA yields `([9,2], [9,2])` where its SSA-level meaning is
    /// `([9,2], [1,2])`. The verifier must reject: checking each argument in
    /// isolation misses it, because the caller never reads `v0` after the call
    /// (the aliased read is the callee's `return` of the sibling argument).
    /// This is the SSA shape `ssa_gen` emitted for `f(&mut x, x)` before the
    /// ownership pass learned to clone a by-value argument that aliases a
    /// `&mut` argument of the same call (noir-lang/noir-claude#1553).
    #[test]
    fn end_to_end_mut_ref_arg_pointing_at_by_value_arg_of_same_call_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = allocate -> &mut [Field; 2]
                store v0 at v1
                v2, v3 = call f1(v1, v0) -> ([Field; 2], [Field; 2])
                return v2, v3
            }
            brillig(inline) fn f f1 {
              b0(v0: &mut [Field; 2], v1: [Field; 2]):
                v2 = load v0 -> [Field; 2]
                v3 = array_set v2, index u32 0, value Field 9
                store v3 at v0
                inc_rc v3
                return v1, v3
            }"#;
        assert_verifier_rejects(src);
    }

    /// Companion of
    /// [`end_to_end_mut_ref_arg_pointing_at_by_value_arg_of_same_call_is_rejected`]
    /// with no reference anywhere: the **same array value occupies two argument
    /// positions** (`call f1(v0, v0)`), the callee mutates one in place at RC 1
    /// and returns the other, corrupted. This pins that the missing relation is
    /// between argument positions as such, not something introduced by the
    /// `&mut` argument. The frontend does not emit this shape (a reused
    /// by-value argument is always cloned), so it is reachable from hand-written
    /// SSA only; it must still be rejected.
    #[test]
    fn end_to_end_same_array_in_two_argument_positions_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1, v2 = call f1(v0, v0) -> ([Field; 2], [Field; 2])
                return v1, v2
            }
            brillig(inline) fn f f1 {
              b0(v0: [Field; 2], v1: [Field; 2]):
                v2 = array_set v0, index u32 0, value Field 9
                inc_rc v2
                return v1, v2
            }"#;
        assert_verifier_rejects(src);
    }

    /// The well-formed counterpart of
    /// [`end_to_end_mut_ref_arg_pointing_at_by_value_arg_of_same_call_is_rejected`]:
    /// the `inc_rc v0` the ownership pass emits for the by-value sibling of a
    /// `&mut` argument is present before the call, so the callee's in-place
    /// write copies (RC is 2) and the by-value parameter keeps its snapshot.
    /// Accepted — this pins that the co-aliased-arguments check credits the
    /// protecting `inc_rc` instead of flagging every call that passes a buffer
    /// through two positions.
    #[test]
    fn end_to_end_mut_ref_arg_pointing_at_protected_by_value_arg_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = allocate -> &mut [Field; 2]
                store v0 at v1
                inc_rc v0
                v2, v3 = call f1(v1, v0) -> ([Field; 2], [Field; 2])
                return v2, v3
            }
            brillig(inline) fn f f1 {
              b0(v0: &mut [Field; 2], v1: [Field; 2]):
                v2 = load v0 -> [Field; 2]
                v3 = array_set v2, index u32 0, value Field 9
                store v3 at v0
                inc_rc v3
                return v1, v3
            }"#;
        assert_verifier_accepts_because(
            src,
            "the by-value sibling of the &mut argument is protected by a preceding inc_rc",
        );
    }

    /// Regression for the `valid_after_pass` fuzzer seed `0x96293a520000f025`.
    /// Two block parameters of a join reach a call as sibling arguments. On the
    /// branch that passes the *same* array in both positions, the frontend's
    /// clones are present as `inc_rc`s in that branch — protection delivered
    /// per path, from a predecessor that does not dominate the join. On the
    /// other branch the two positions carry distinct fresh arrays and need no
    /// protection. Well-formed on every path, so the co-aliased-arguments
    /// check must accept: a dominance-only `inc_rc` search rejects this SSA.
    #[test]
    fn end_to_end_join_passing_shared_array_with_branch_local_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                v2 = make_array [Field 1, Field 2] : [Field; 2]
                v3 = make_array [Field 3, Field 4] : [Field; 2]
                jmp b3(v2, v3)
              b2():
                v4 = make_array [Field 5, Field 6] : [Field; 2]
                inc_rc v4
                inc_rc v4
                jmp b3(v4, v4)
              b3(v5: [Field; 2], v6: [Field; 2]):
                v7, v8 = call f1(v5, v6) -> ([Field; 2], [Field; 2])
                return v7, v8
            }
            brillig(inline) fn f f1 {
              b0(v0: [Field; 2], v1: [Field; 2]):
                v2 = array_set v0, index u32 0, value Field 9
                inc_rc v2
                return v1, v2
            }"#;
        assert_verifier_accepts_because(
            src,
            "the only path passing one buffer through both positions bumps it in that branch",
        );
    }

    /// The unprotected counterpart of
    /// [`end_to_end_join_passing_shared_array_with_branch_local_inc_rc_is_accepted`]:
    /// the branch that passes the same array through both positions carries no
    /// `inc_rc`, so on that path the callee's in-place mutation is observable
    /// through the sibling argument. Must be rejected.
    #[test]
    fn end_to_end_join_passing_shared_array_without_inc_rc_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                v2 = make_array [Field 1, Field 2] : [Field; 2]
                v3 = make_array [Field 3, Field 4] : [Field; 2]
                jmp b3(v2, v3)
              b2():
                v4 = make_array [Field 5, Field 6] : [Field; 2]
                jmp b3(v4, v4)
              b3(v5: [Field; 2], v6: [Field; 2]):
                v7, v8 = call f1(v5, v6) -> ([Field; 2], [Field; 2])
                return v7, v8
            }
            brillig(inline) fn f f1 {
              b0(v0: [Field; 2], v1: [Field; 2]):
                v2 = array_set v0, index u32 0, value Field 9
                inc_rc v2
                return v1, v2
            }"#;
        assert_verifier_rejects(src);
    }

    /// A join whose branches pass one array through *different* positions —
    /// `(v2, v3)` on one arm, `(v4, v2)` on the other. `v2` reaches both
    /// argument positions, but never both on the same path, so no path hands
    /// the callee one buffer twice and there is nothing to protect: each
    /// branch-local use of `v2` is that branch's last use and is legitimately
    /// moved without a clone. The check must accept — relating the positions'
    /// backward alias sets without path sensitivity rejects this SSA.
    #[test]
    fn end_to_end_join_passing_array_through_different_positions_per_branch_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1):
                v2 = make_array [Field 1, Field 2] : [Field; 2]
                jmpif v0 then: b1(), else: b2()
              b1():
                v3 = make_array [Field 3, Field 4] : [Field; 2]
                jmp b3(v2, v3)
              b2():
                v4 = make_array [Field 5, Field 6] : [Field; 2]
                jmp b3(v4, v2)
              b3(v5: [Field; 2], v6: [Field; 2]):
                v7, v8 = call f1(v5, v6) -> ([Field; 2], [Field; 2])
                return v7, v8
            }
            brillig(inline) fn f f1 {
              b0(v0: [Field; 2], v1: [Field; 2]):
                v2 = array_set v0, index u32 0, value Field 9
                inc_rc v2
                return v1, v2
            }"#;
        assert_verifier_accepts_because(
            src,
            "no single path passes the same buffer through two argument positions",
        );
    }
}
