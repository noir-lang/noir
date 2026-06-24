//! The `array_set` reachable-aliased-use check.
//!
//! Iterates every `array_set` in each Brillig function and rejects the SSA
//! when the in-place mutation could be observed through an aliased read on a
//! forward path. The aliasing/coverage/forward-walk machinery it drives lives
//! in the parent [`super`] module.

use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{function::Function, instruction::Instruction},
        ssa_gen::Ssa,
    },
};

use super::Context;

/// Verifies the `array_set` / `inc_rc` aliasing invariant on every Brillig
/// function in `ssa`. See the [`super`] module docs for details.
///
/// The entire module containing this function is gated behind
/// `#[cfg(debug_assertions)]`, so it is a no-op (and absent at the linker
/// level) in release builds — see the pipeline wiring in
/// [`crate::ssa::primary_passes`].
pub(crate) fn verify(ssa: &Ssa) -> RtResult<()> {
    for function in ssa.functions.values() {
        verify_function(function)?;
    }
    Ok(())
}

/// Per-function entry point. Skips ACIR functions (the invariant only
/// applies to Brillig, where `array_set` may mutate in place) and runs the
/// alias / dominance / reachable-use checks for every `array_set` in the
/// function.
fn verify_function(function: &Function) -> RtResult<()> {
    if !function.runtime().is_brillig() {
        return Ok(());
    }

    let ctx = Context::new(function);

    for block_id in function.reachable_blocks() {
        for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            let Instruction::ArraySet { array, index: write_index, .. } =
                function.dfg[*instruction_id]
            else {
                continue;
            };

            let alias_set = ctx.alias_set_for(array, block_id, idx);

            // Narrow the alias-set to the members that are *not* provably
            // protected on every path to this `array_set`. An empty result
            // means every member is RC-bumped or freshly allocated before the
            // array_set on all paths, so the in-place mutation is never
            // observable — accept without the forward walk. A covered member
            // (e.g. an `inc_rc`'d sibling) is dropped so the forward walk does
            // not falsely flag a read of storage the array_set never mutates.
            let use_set = ctx.unprotected_aliases(&alias_set, array, block_id, idx);
            if use_set.is_empty() {
                continue;
            }

            // The array_set's index, if constant — lets the walk skip
            // `array_get`s on disjoint constant indices (the pedersen-style
            // pattern). A dynamic write index means we must conservatively
            // flag every aliased read.
            let write_index_const = function.dfg.get_numeric_constant(write_index);

            // Expensive: forward CFG walk looking for an aliased read.
            // A hit means the `array_set` may mutate storage in place
            // (RC=1) and a downstream instruction will observe the
            // pre-mutation contents through an aliased name.
            if let Some(hit) = ctx.find_reachable_aliased_use(
                &use_set,
                array,
                *instruction_id,
                block_id,
                idx,
                write_index_const,
            ) {
                let call_stack = function.dfg.get_instruction_call_stack(*instruction_id);
                let aliased_use_call_stack =
                    function.dfg.get_instruction_call_stack(hit.instruction);
                let message = format!(
                    "array_set in function {} on array {array} has an aliased read of {} on a \
                     forward path with no preceding `inc_rc`; the in-place mutation would be \
                     observable through that alias",
                    function.name(),
                    hit.value,
                );
                return Err(RuntimeError::ArraySetAliasViolation {
                    message,
                    call_stack,
                    aliased_use_call_stack,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;

    use super::super::tests::{assert_verifier_accepts, assert_verifier_accepts_because};
    use super::Context;
    use crate::ssa::{
        ir::{
            basic_block::BasicBlockId,
            function::Function,
            instruction::{Instruction, InstructionId},
            value::ValueId,
        },
        ssa_gen::Ssa,
    };

    /// Parse `src`, run the `array_set` verifier, and require it to reject the
    /// SSA with an [`crate::errors::RuntimeError::ArraySetAliasViolation`].
    /// Panics on any other outcome. Runs `array_set::verify` directly (not the
    /// combined check) so the assertion proves the *array_set* verifier is the
    /// one that caught the hazard.
    fn assert_verifier_rejects(src: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        let err = super::verify(&ssa).expect_err("expected the verifier to reject");
        assert!(
            matches!(err, crate::errors::RuntimeError::ArraySetAliasViolation { .. }),
            "expected ArraySetAliasViolation, got {err:?}",
        );
    }
    /// A read value that carries its own `inc_rc` **and** is a loop
    /// back-edge participant is **still a hazard** when it merely
    /// *receives* the source's storage on a forward edge. Here `array_set s`
    /// (b1) mutates `s` in place, then `p := s` (b1 → b2) and `p` is read by
    /// the `call` in b2. `p` has an `inc_rc` and is a back-edge arg (b3 → b2),
    /// but the `inc_rc` runs *after* the mutation and `p` flows *out of* the
    /// source (`p ← s`), not into it — so the bump can't protect the
    /// `array_set`. The read observes the in-place mutation; the verifier must
    /// reject.
    ///
    /// This pins the soundness boundary of the protected-participant filter:
    /// it excludes a value only when that value is in the *source's*
    /// alias-set (flows *into* the source, so its `inc_rc` is loop-carried
    /// before the mutation).
    #[test]
    fn end_to_end_forward_threaded_read_with_inc_rc_participant_is_rejected() {
        let src = r#"
            brillig(inline) fn f f0 {
              b0(v0: u1, vs: [u32; 1]):
                jmp b1(vs)
              b1(s: [u32; 1]):
                v1 = array_set s, index u32 0, value u32 9
                jmp b2(s)
              b2(p: [u32; 1]):
                inc_rc p
                v2 = call f0(v0, p) -> u1
                jmpif v0 then: b3(), else: b4()
              b3():
                jmp b2(p)
              b4():
                jmp b1(v1)
            }"#;
        assert_verifier_rejects(src);
    }

    /// Inclusive-range (`..=`) peel, accepted by the index-aware
    /// `array_set`-use rule. This is the SSA the frontend emits for
    ///
    /// ```ignore
    /// for _ in 254_u8..=255_u8 { c4[0] = 9; c4 = c3; c3 = [b[0]]; }
    /// ```
    ///
    /// An inclusive range lowers to an exclusive loop plus a duplicated
    /// **peel** of the final iteration (see `codegen_for` in `ssa_gen`).
    /// Here `b1`/`b2` are the loop and `b4` is the peel; the loop variable
    /// `v29` (`c4`) is `array_set` in *both*, at the **same** constant
    /// index `0`. The forward walk from the loop's `array_set v29` (`b2`)
    /// reaches the peel's `array_set v29` (`b4`) — but that write
    /// *overwrites* index `0`, the only tainted index, so it can't observe
    /// the loop's in-place mutation. Index-aware handling of `array_set`
    /// uses (the same `tainted`-index test the `array_get` rule uses)
    /// therefore accepts it, matching the runtime: the program executes
    /// identically under Brillig and comptime.
    ///
    /// This is *not* peel detection — there's no reliable post-`mem2reg`
    /// marker for a peel block. It's a precise consequence of `array_set`
    /// semantics (a write observes only the indices it doesn't overwrite),
    /// so it also covers the corresponding `..` case and any other shape
    /// where the aliased write hits the same index. A peel whose duplicated
    /// body *reads* a tainted index some other way (e.g. an `array_get` or
    /// a `call`) is still flagged.
    #[test]
    fn end_to_end_inclusive_range_peel_array_set_same_index_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v1 = make_array [u8 1] : [u8; 1]
                v4 = make_array [u8 2] : [u8; 1]
                v7 = make_array [u8 3] : [u8; 1]
                jmp b1(u8 254, v1, v4)
              b1(v10: u8, v28: [u8; 1], v29: [u8; 1]):
                v13 = lt v10, u8 255
                jmpif v13 then: b2(), else: b3()
              b2():
                v18 = array_set v29, index u32 0, value u8 9
                v20 = make_array [u8 3] : [u8; 1]
                v21 = unchecked_add v10, u8 1
                jmp b1(v21, v20, v28)
              b3():
                jmpif u1 1 then: b4(), else: b5(v28, v29)
              b4():
                v25 = array_set v29, index u32 0, value u8 9
                v27 = make_array [u8 3] : [u8; 1]
                jmp b5(v27, v28)
              b5(v30: [u8; 1], v31: [u8; 1]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "the peel's array_set v29 (b4) overwrites index 0 — the only index the loop's \
             array_set v29 (b2) tainted — so it observes no in-place mutation",
        );
    }

    /// Multi-index inclusive-range (`..=`) peel with the swap `c4 = c3`.
    /// SSA for
    ///
    /// ```ignore
    /// for _ in 254_u8..=255_u8 { c4[0] = 9; c4[1] = 19; c4 = c3; c3 = [b[0], b[1]]; }
    /// ```
    ///
    /// Same peel shape as
    /// [`Self::end_to_end_inclusive_range_peel_array_set_same_index_is_accepted`],
    /// but the loop body writes **both** indices, so the index-aware
    /// `array_set`-use rule alone can't rescue it (the peel's single-index
    /// write copies the other tainted index forward). It is accepted by the
    /// **swap exclusion** instead: on the back-edge `jmp b1(v28, v27, v37)`
    /// the source param `v38` (`c4`) is rebound to its sibling `v37` (`c3`),
    /// whose own back-edge arg `v27` is an iteration-local `make_array` — so
    /// `v37` is a distinct per-iteration storage and is dropped from
    /// `backward(v38)`. With `v37` gone, the walk kills `v38` on the
    /// back-edge, and the peel's `array_set v38` (`b4`) — which sources the
    /// same header param — never sees it in the use-set. The loop body
    /// mutates one storage while the peel mutates a different one and both
    /// writes are dead; Brillig and comptime agree.
    #[test]
    fn end_to_end_multi_index_inclusive_range_peel_swap_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v2 = make_array [u8 1, u8 11] : [u8; 2]
                v6 = make_array [u8 2, u8 12] : [u8; 2]
                v10 = make_array [u8 3, u8 13] : [u8; 2]
                jmp b1(u8 254, v2, v6)
              b1(v13: u8, v37: [u8; 2], v38: [u8; 2]):
                v16 = lt v13, u8 255
                jmpif v16 then: b2(), else: b3()
              b2():
                v22 = array_set v38, index u32 0, value u8 9
                v25 = array_set v22, index u32 1, value u8 19
                v27 = make_array [u8 3, u8 13] : [u8; 2]
                v28 = unchecked_add v13, u8 1
                jmp b1(v28, v27, v37)
              b3():
                jmpif u1 1 then: b4(), else: b5(v37, v38)
              b4():
                v32 = array_set v38, index u32 0, value u8 9
                v34 = array_set v32, index u32 1, value u8 19
                v36 = make_array [u8 3, u8 13] : [u8; 2]
                jmp b5(v36, v37)
              b5(v39: [u8; 2], v40: [u8; 2]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "the back-edge swap v38 ← v37 with v37 freshened by the iteration-local \
             make_array v27 drops v37 from v38's alias-set, so the walk kills v38 on the \
             back-edge and the peel's array_set v38 (b4) — sourcing the same header param — \
             is not flagged",
        );
    }

    /// The general form (no peel): a swap `c4 = c3` followed by a
    /// whole-array read. SSA for
    ///
    /// ```ignore
    /// for _ in 253_u8..255_u8 { c4[0] = 9; c4[1] = 19; c4 = c3; c3 = [b[0], b[1]]; }
    /// println(c4);
    /// ```
    ///
    /// This is an **exclusive** (`..`) loop — *no peel* — exercising the
    /// swap exclusion directly. On the back-edge `jmp b1(v27, v26, v30)`
    /// the source param `v31` (`c4`) is rebound to its sibling `v30` (`c3`),
    /// whose own back-edge arg `v26` is an iteration-local `make_array`, so
    /// `v30` is dropped from `backward(v31)`. The walk then kills `v31` on
    /// the back-edge, and the loop-exit `call f1(v31)` (`b3`, the `println`)
    /// reads a value no longer in the use-set. The loop's `array_set v31`
    /// mutates `c4`'s storage, but its result is discarded and `v31` is
    /// rebound to a fresh `c3`-derived array — the mutated storage is dead,
    /// and a whole-array read of the swapped-in value observes nothing.
    /// There is no `inc_rc` (the frontend correctly omitted it — the
    /// storages are genuinely distinct); Brillig and comptime agree.
    #[test]
    fn end_to_end_loop_swap_then_whole_array_read_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v2 = make_array [u8 1, u8 11] : [u8; 2]
                v6 = make_array [u8 2, u8 12] : [u8; 2]
                jmp b1(u8 253, v2, v6)
              b1(v13: u8, v30: [u8; 2], v31: [u8; 2]):
                v14 = lt v13, u8 255
                jmpif v14 then: b2(), else: b3()
              b2():
                v21 = array_set v31, index u32 0, value u8 9
                v24 = array_set v21, index u32 1, value u8 19
                v26 = make_array [u8 3, u8 13] : [u8; 2]
                v27 = unchecked_add v13, u8 1
                jmp b1(v27, v26, v30)
              b3():
                call f1(v31)
                return
            }
            brillig(inline) fn observe f1 {
              b0(v0: [u8; 2]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "the back-edge swap v31 ← v30 with v30 freshened by the iteration-local \
             make_array v26 drops v30 from v31's alias-set, so the walk kills v31 on the \
             back-edge and the loop-exit whole-array read call f1(v31) is not flagged",
        );
    }

    /// **Swap-exclusion soundness canary — freshening guard.** The swap
    /// `v3 ← v2` (`P ← Q`) is present on the back-edge, but `v2`'s own
    /// back-edge arg is `v2` itself (loop-**invariant** `c3`, not a fresh
    /// `make_array`). Then `P_k = Q_{k-1} = Q_0 = Q_k`, so `P` and `Q`
    /// genuinely share storage from the first swap on: the in-loop
    /// `array_set v3` mutates it and `array_get v2` observes the mutation.
    /// The exclusion must **not** fire (its freshening precondition fails),
    /// and the verifier must reject. Guards against dropping `v2` purely
    /// because it's swapped into the source.
    #[test]
    fn end_to_end_swap_with_loop_invariant_sibling_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [u8 1] : [u8; 1]
                v1 = make_array [u8 2] : [u8; 1]
                jmp b1(u8 0, v0, v1)
              b1(v10: u8, v2: [u8; 1], v3: [u8; 1]):
                v12 = lt v10, u8 5
                jmpif v12 then: b2(), else: b3()
              b2():
                v15 = array_set v3, index u32 0, value u8 9
                v17 = array_get v2, index u32 0 -> u8
                constrain v17 == u8 1
                v18 = unchecked_add v10, u8 1
                jmp b1(v18, v2, v2)
              b3():
                return
            }"#;
        assert_verifier_rejects(src);
    }

    /// **Swap-exclusion soundness canary — loop-entry guard.** The swap
    /// `v3 ← v2` with `v2` freshened by an iteration-local `make_array`
    /// (`v9`) *does* satisfy the freshening precondition, but the
    /// pre-header feeds the **same** array `v0` to both header params
    /// (`jmp b1(.., v0, v0)`), so `P_0 = Q_0 = v0`: at the entry
    /// iteration the `array_set v3` mutates the storage that `array_get
    /// v2` then reads. The directed backward walk keeps `v2` and `v3` in
    /// separate sets, so without the loop-entry guard the exclusion would
    /// fire and mask this hazard. It must **not** fire; the verifier must
    /// reject.
    #[test]
    fn end_to_end_swap_with_sibling_same_value_entry_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [u8 1] : [u8; 1]
                jmp b1(u8 0, v0, v0)
              b1(v10: u8, v2: [u8; 1], v3: [u8; 1]):
                v12 = lt v10, u8 5
                jmpif v12 then: b2(), else: b3()
              b2():
                v15 = array_set v3, index u32 0, value u8 9
                v17 = array_get v2, index u32 0 -> u8
                constrain v17 == u8 1
                v9 = make_array [u8 3] : [u8; 1]
                v18 = unchecked_add v10, u8 1
                jmp b1(v18, v9, v2)
              b3():
                return
            }"#;
        assert_verifier_rejects(src);
    }

    /// The swap freshening accepts a `Call` result, not only a
    /// `make_array`. Same shape as
    /// [`Self::end_to_end_loop_swap_then_whole_array_read_is_accepted`],
    /// but the sibling `v2` (`c3`) is freshened by `v20 = call f1()`
    /// (`c3 = f()`) rather than a literal `make_array`. A `Call` result is
    /// a fresh per-iteration allocation (the same assumption the
    /// non-aliasing filter makes), so `v2` is dropped from `v3`'s
    /// alias-set, the walk kills `v3` on the back-edge, and the loop-exit
    /// `call f2(v3)` is not flagged.
    #[test]
    fn end_to_end_swap_freshened_by_call_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [u8 1] : [u8; 1]
                v1 = make_array [u8 2] : [u8; 1]
                jmp b1(u8 0, v0, v1)
              b1(v10: u8, v2: [u8; 1], v3: [u8; 1]):
                v12 = lt v10, u8 5
                jmpif v12 then: b2(), else: b3()
              b2():
                v15 = array_set v3, index u32 0, value u8 9
                v20 = call f1() -> [u8; 1]
                v18 = unchecked_add v10, u8 1
                jmp b1(v18, v20, v2)
              b3():
                call f2(v3)
                return
            }
            brillig(inline) fn alloc f1 {
              b0():
                v0 = make_array [u8 7] : [u8; 1]
                return v0
            }
            brillig(inline) fn observe f2 {
              b0(v0: [u8; 1]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "v2 is freshened on the back-edge by a Call result (v20), a fresh per-iteration \
             allocation, so the swap exclusion drops v2 from v3's alias-set and the loop-exit \
             read call f2(v3) is not flagged",
        );
    }

    /// **Swap-exclusion soundness canary — `array_set` results are not
    /// fresh.** The sibling `v2` (`c3`) is freshened on the back-edge by
    /// `v20 = array_set v2, …` — an `array_set` result, which may be an
    /// **in-place** mutation rather than a new allocation. So
    /// `v2_k = v2_{k-1}`'s storage, and after the swap `v3` aliases it: the
    /// in-loop `array_set v3` mutates the storage that `array_get v2` then
    /// reads. The exclusion must **not** fire (an `array_set` result is
    /// excluded from `iteration_local_fresh`), and the verifier must
    /// reject. Guards against widening the freshening to all
    /// non-aliasing results.
    #[test]
    fn end_to_end_swap_freshened_by_array_set_result_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [u8 1] : [u8; 1]
                v1 = make_array [u8 2] : [u8; 1]
                jmp b1(u8 0, v0, v1)
              b1(v10: u8, v2: [u8; 1], v3: [u8; 1]):
                v12 = lt v10, u8 5
                jmpif v12 then: b2(), else: b3()
              b2():
                v15 = array_set v3, index u32 0, value u8 9
                v17 = array_get v2, index u32 0 -> u8
                constrain v17 == u8 1
                v20 = array_set v2, index u32 0, value u8 5
                v18 = unchecked_add v10, u8 1
                jmp b1(v18, v20, v2)
              b3():
                return
            }"#;
        assert_verifier_rejects(src);
    }

    /// ACIR functions are skipped: `inc_rc` / `dec_rc` are no-ops in ACIR and
    /// `array_set` always produces a fresh array.
    #[test]
    fn acir_function_is_skipped() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 0 -> u32
                return v5
            }"#;
        assert_verifier_accepts(src);
    }

    /// End-to-end: the user's well-formed example from the design
    /// discussion. The loop mutates `v2` in place each iteration and
    /// threads the result back through the block-parameter, so no
    /// `inc_rc` is needed and the verifier must accept.
    #[test]
    fn end_to_end_well_formed_loop_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = array_get v2, index u32 0 -> u32
                v8 = eq v3, u32 1
                jmpif v8 then: b4(), else: b5()
              b3():
                return
              b4():
                v10 = eq v6, u32 99
                constrain v6 == u32 99
                jmp b5()
              b5():
                v11 = array_set v2, index u32 0, value u32 99
                v12 = add v3, u32 1
                jmp b1(v11, v12)
            }"#;
        assert_verifier_accepts(src);
    }

    /// End-to-end: PR-12671 malformed repro. `array_get v0` reads the
    /// pre-header value while `array_set v2` mutates the same storage in
    /// place — verifier must reject.
    #[test]
    fn end_to_end_pr_12671_repro_is_rejected() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        assert_verifier_rejects(src);
    }

    /// End-to-end: same PR-12671 SSA but with an `inc_rc v0` placed in the
    /// pre-header. `inc_rc v0` dominates the `array_set`, so the check
    /// short-circuits and the verifier must accept.
    #[test]
    fn end_to_end_pr_12671_repro_with_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                inc_rc v0
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        assert_verifier_accepts(src);
    }

    /// A branch-local `inc_rc` does **not** protect an `array_set` reached on a
    /// path that skips it. `inc_rc v1` lives only on the `then` path (b1), but
    /// the `array_set v1` in the join block b3 is also reachable via b2, which
    /// has no `inc_rc`. On the `v0 = false` path `v1` has RC 1, so the
    /// `array_set` mutates it in place and the following `array_get v1` observes
    /// the mutation. The verifier must reject — a non-dominating `inc_rc`
    /// cannot vouch for the `array_set`.
    #[test]
    fn end_to_end_branch_local_inc_rc_does_not_protect_array_set_in_join_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1, v1: [Field; 1]):
                jmpif v0 then: b1(), else: b2()
              b1():
                inc_rc v1
                jmp b3()
              b2():
                jmp b3()
              b3():
                v2 = array_set v1, index u32 0, value Field 7
                v3 = array_get v1, index u32 0 -> Field
                return v3
            }"#;
        assert_verifier_rejects(src);
    }

    /// Branch-local `inc_rc`s that **collectively** cover every path do
    /// protect the `array_set`, even when no single one dominates it. Both
    /// arms of the diamond (b1 *and* b2) bump `v1` before the join block b3
    /// holds `array_set v1` and a read of `v1`. Neither `inc_rc` dominates b3
    /// on its own, but every path to the `array_set` passes through one of
    /// them, so `v1` has RC ≥ 2 on every path and the `array_set` always
    /// copies. The verifier must accept.
    ///
    /// This is the minimized crux of a `comptime_vs_brillig_direct` fuzzer
    /// case: the frontend emits one `inc_rc` per branch feeding a join, which
    /// a single-block dominance check would wrongly reject. It is the exact
    /// counterpart of
    /// `end_to_end_branch_local_inc_rc_does_not_protect_array_set_in_join_is_rejected`
    /// — adding the second arm's `inc_rc` is what flips it from a hazard to
    /// safe.
    #[test]
    fn end_to_end_branch_local_inc_rcs_on_all_arms_protect_array_set_in_join_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1, v1: [Field; 1]):
                jmpif v0 then: b1(), else: b2()
              b1():
                inc_rc v1
                jmp b3()
              b2():
                inc_rc v1
                jmp b3()
              b3():
                v2 = array_set v1, index u32 0, value Field 7
                v3 = array_get v1, index u32 0 -> Field
                return v3
            }"#;
        assert_verifier_accepts_because(
            src,
            "inc_rc v1 on both arms collectively cover every path to the array_set; \
             neither dominates b3 alone, but together they form a cut",
        );
    }

    /// Path-specific protection mixing `inc_rc` and freshness across a loop.
    /// The loop body block b5 holds `array_set v7` on its parameter and reads
    /// `v7` (on the next iteration, via the back-edge). `v7` is fed by two
    /// arms: b2 passes the loop-invariant `v1` after an `inc_rc v1`, while b3
    /// passes a freshly allocated `make_array`. On every path the storage
    /// `array_set v7` may mutate is either RC-bumped (`v1` via b2) or freshly
    /// allocated this iteration (`v4` via b3), so no read ever observes an
    /// in-place mutation through an alias. The verifier must accept.
    ///
    /// This is the minimized crux of a `comptime_vs_brillig_direct` fuzzer
    /// case. It is the reason a coverage check that only recognizes `inc_rc`
    /// blocks is insufficient: the b3 arm carries no `inc_rc`, only a fresh
    /// allocation, so freshness must count as protection on that path.
    #[test]
    fn end_to_end_loop_param_protected_by_inc_rc_and_freshness_on_alternate_arms_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1, v1: [Field; 1]):
                jmp b1(u32 0)
              b1(v2: u32):
                v3 = eq v2, u32 5
                jmpif v3 then: b6(), else: b4()
              b4():
                jmpif v0 then: b2(), else: b3()
              b2():
                inc_rc v1
                jmp b5(v1)
              b3():
                v4 = make_array [Field 0] : [Field; 1]
                jmp b5(v4)
              b5(v7: [Field; 1]):
                v8 = array_get v7, index u32 0 -> Field
                v9 = array_set v7, index u32 0, value Field 7
                v10 = add v2, u32 1
                jmp b1(v10)
              b6():
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "every path to array_set v7 either bumps RC (inc_rc v1 on the b2 arm) or \
             allocates fresh storage (make_array on the b3 arm)",
        );
    }

    /// A covered alias that flows into the source must be dropped from the
    /// forward walk's use-set, not flagged. `v3` (the `array_set` source) is fed
    /// by `v2` on the b1 arm — where `v2` is `inc_rc`'d, so on that arm the
    /// `array_set` copies and `v2` is never mutated — and by the function
    /// parameter `v1` on the b2 arm (uncovered). The later `array_get v2` reads
    /// the *covered* sibling: on the arm where `v3 = v2` the bump protects it,
    /// and on the arm where `v3 = v1` the `array_set` mutates `v1`, not `v2`. So
    /// the read is never a hazard and the verifier must accept — even though
    /// `v2` is in the source's alias-set and is read after the `array_set`.
    ///
    /// This is the minimized crux of a `comptime_vs_brillig_direct` fuzzer
    /// case. Seeding the forward walk with the *whole* alias-set (including the
    /// covered `v2`) would falsely flag the `array_get v2`; keeping only the
    /// uncovered members (`v3`/`v1`) avoids it. The source's own b2 arm stays
    /// uncovered, so `some_inc_rc_precedes` does not accept the `array_set`
    /// outright — the per-member coverage is what makes the difference.
    #[test]
    fn end_to_end_inc_rc_covered_alias_read_after_array_set_is_dropped_and_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1, v1: [Field; 1], v2: [Field; 1]):
                jmpif v0 then: b1(), else: b2()
              b1():
                inc_rc v2
                jmp b3(v2)
              b2():
                jmp b3(v1)
              b3(v3: [Field; 1]):
                v5 = array_set v3, index u32 0, value Field 7
                v6 = array_get v2, index u32 0 -> Field
                return v6
            }"#;
        assert_verifier_accepts_because(
            src,
            "the read is of v2, an inc_rc-covered alias of the source; only the \
             uncovered members (v3/v1) belong in the forward walk's use-set",
        );
    }

    /// Swap exclusion must reach an inner-loop `array_set` seeded from the
    /// swapped outer-loop parameter. The outer loop (header b1) rotates its
    /// array params on the back-edge — `a ← b` while `b ← make_array` — so the
    /// swap filter records `b` (v4) as excluded from `a` (v3): they are
    /// distinct per-iteration storage. The inner loop's parameter `c` (v7) is
    /// seeded from `a` on the forward edge b2→b3, and the inner body does
    /// `array_set c` and then reads `b`. Because `c = a` within the iteration,
    /// `b` is distinct from `c` too, so the read is never a hazard. The
    /// verifier must accept — which requires the swap exclusion on `a` to
    /// propagate forward onto `c`.
    ///
    /// This is the minimized crux of a `comptime_vs_brillig_direct` fuzzer
    /// false-positive (confirmed valid via `--release`): without the forward
    /// propagation, `b` stays in `c`'s alias-set and the `array_get b` is
    /// wrongly flagged.
    #[test]
    fn end_to_end_swap_exclusion_propagates_to_inner_loop_source_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: u1):
                v1 = make_array [Field 0] : [Field; 1]
                v2 = make_array [Field 1] : [Field; 1]
                jmp b1(v1, v2, u32 0)
              b1(v3: [Field; 1], v4: [Field; 1], v5: u32):
                v6 = lt v5, u32 3
                jmpif v6 then: b2(), else: b8()
              b2():
                jmp b3(v3, u32 0)
              b3(v7: [Field; 1], v8: u32):
                v9 = lt v8, u32 3
                jmpif v9 then: b4(), else: b6()
              b4():
                v10 = array_set v7, index u32 0, value Field 7
                v11 = array_get v4, index u32 0 -> Field
                v12 = add v8, u32 1
                jmp b3(v10, v12)
              b6():
                v14 = make_array [Field 2] : [Field; 1]
                v15 = add v5, u32 1
                jmp b1(v4, v14, v15)
              b8():
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "b (v4) is swap-excluded from a (v3); a is forwarded to the inner-loop \
             source c (v7), so the exclusion must propagate and the array_get v4 is safe",
        );
    }

    /// The linear analog of `make_array_with_forward_read` but via a `Call`
    /// result: `v2 = v1 = v0` is the same fresh call-allocated storage,
    /// `array_set v2` mutates it in place, and `array_get v1` reads it back —
    /// a genuine hazard. A `Call` result must NOT be blanket-credited as
    /// distinct-per-iteration freshness (it is one-time storage here), so the
    /// verifier must reject, exactly as it does for the `make_array` analog.
    #[test]
    fn end_to_end_call_result_aliased_via_forward_block_param_with_forward_read_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = call f1() -> [Field; 1]
                jmp b1(v0)
              b1(v1: [Field; 1]):
                jmp b2(v1)
              b2(v2: [Field; 1]):
                v3 = array_set v2, index u32 0, value Field 7
                v4 = array_get v1, index u32 0 -> Field
                return v4
            }
            brillig(inline) fn alloc f1 {
              b0():
                v0 = make_array [Field 0] : [Field; 1]
                return v0
            }"#;
        assert_verifier_rejects(src);
    }

    /// Index-aware filter: a constant-index `array_set` followed by a
    /// constant-index `array_get` on the same alias at a **different**
    /// index is safe. In-place mutation at one position doesn't affect
    /// reads at another, so the verifier should accept.
    #[test]
    fn end_to_end_array_set_array_get_disjoint_constant_indices_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 1 -> u32
                return v5
            }"#;
        assert_verifier_accepts_because(
            src,
            "array_set at idx 0 + array_get at idx 1 access disjoint positions; not a hazard",
        );
    }

    /// Counterpart to the disjoint case: matching constant indices mean
    /// the read observes the in-place mutation, so the verifier must
    /// reject.
    #[test]
    fn end_to_end_array_set_array_get_matching_constant_indices_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 0 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// A **dynamic** write index could touch any position, so the filter
    /// can't prove disjointness; the verifier must conservatively reject
    /// any aliased read (even one at a known disjoint-looking constant
    /// index).
    #[test]
    fn end_to_end_array_set_dynamic_index_with_array_get_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                v3 = array_set v0, index v1, value u32 99
                v5 = array_get v0, index u32 1 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// Symmetric to the previous case: write index is constant but the
    /// read's index is dynamic. The runtime read could land on the
    /// write's position, so the verifier conservatively rejects.
    #[test]
    fn end_to_end_array_set_constant_with_dynamic_array_get_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index v1 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// A second `array_set` on the alias at a **different** constant index
    /// is a hazard: it produces a copy of the source, so it observes
    /// (copies forward) the source's tainted index `0` at its non-written
    /// positions. The index-aware `array_set`-use rule flags it precisely
    /// because the write index `1` differs from the tainted index `0`.
    /// (Compare
    /// [`Self::end_to_end_inclusive_range_peel_array_set_same_index_is_accepted`],
    /// where the write hits the same index and is accepted.)
    #[test]
    fn end_to_end_array_set_followed_by_another_array_set_on_alias_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_set v0, index u32 1, value u32 88
                v7 = array_get v5, index u32 0 -> u32
                return v7
            }"#;
        assert_verifier_rejects(src);
    }

    /// Counterpart to the different-index case: a second `array_set` on the
    /// alias at the **same** constant index as the source overwrites the
    /// only tainted index, so it observes none of the source's in-place
    /// mutation and is accepted — the `array_set`-use analogue of the
    /// disjoint-`array_get` rule. At runtime `v3`'s write to index 0 is
    /// dead (overwritten by `v5`), and `v7` reads `v5`'s result.
    #[test]
    fn end_to_end_array_set_followed_by_another_array_set_same_index_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_set v0, index u32 0, value u32 88
                v7 = array_get v5, index u32 0 -> u32
                return v7
            }"#;
        assert_verifier_accepts_because(
            src,
            "the second array_set on v0 writes the same index 0 the first tainted, overwriting \
             it rather than observing the in-place mutation",
        );
    }

    /// Chain of `array_set`s on the same backing storage where the read
    /// is at an index hit by a *later* link in the chain. The first
    /// `array_set`'s write index alone is disjoint from the read, so the
    /// pre-chain-aware filter would have skipped — but a downstream
    /// chain link writes the read's index on the same storage, so the
    /// read does observe the in-place mutation. The chain-aware filter
    /// must reject.
    #[test]
    fn end_to_end_chain_taints_downstream_index_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 3]):
                v2 = array_set v0, index u32 0, value u32 10
                v4 = array_set v2, index u32 1, value u32 20
                v6 = array_set v4, index u32 2, value u32 30
                v8 = array_get v0, index u32 1 -> u32
                return v8
            }"#;
        assert_verifier_rejects(src);
    }

    /// Variant of the chain hazard with an `inc_rc` on the *source*
    /// placed *after* the chain. The `inc_rc` cannot undo the damage that
    /// the in-place chain writes have already done to `v0`'s storage;
    /// `tainted_indices` survives the `inc_rc` and the read at the
    /// tainted index is still a hazard.
    #[test]
    fn end_to_end_chain_with_late_inc_rc_on_source_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 3]):
                v2 = array_set v0, index u32 0, value u32 10
                v4 = array_set v2, index u32 1, value u32 20
                v6 = array_set v4, index u32 2, value u32 30
                inc_rc v0
                v8 = array_get v0, index u32 1 -> u32
                return v8
            }"#;
        assert_verifier_rejects(src);
    }

    /// Variant of the chain hazard with `inc_rc` on the *last* chain link
    /// placed before the read. The `inc_rc` still doesn't help — by the
    /// time it runs, the chain has already in-place mutated the storage
    /// at index 1 of `v0`. `tainted_indices` survives.
    #[test]
    fn end_to_end_chain_with_late_inc_rc_on_chain_tail_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 3]):
                v2 = array_set v0, index u32 0, value u32 10
                v4 = array_set v2, index u32 1, value u32 20
                v6 = array_set v4, index u32 2, value u32 30
                inc_rc v6
                v8 = array_get v0, index u32 1 -> u32
                return v8
            }"#;
        assert_verifier_rejects(src);
    }

    /// Mirror of [`end_to_end_chain_taints_downstream_index_is_rejected`]
    /// where the read sits at an index that *no* chain write touched.
    /// `tainted_indices` accumulates to `{0, 1}` and the read at index 2
    /// remains safely disjoint.
    #[test]
    fn end_to_end_chain_with_read_at_untouched_index_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 3]):
                v2 = array_set v0, index u32 0, value u32 10
                v4 = array_set v2, index u32 1, value u32 20
                v6 = array_get v0, index u32 2 -> u32
                return v6
            }"#;
        assert_verifier_accepts_because(
            src,
            "chain writes idx 0 and 1; read at idx 2 is untouched",
        );
    }

    /// Mid-chain `inc_rc` on a chain link clears `derived`: subsequent
    /// chain writes run on fresh storage and so don't taint the source.
    /// The post-inc_rc write at index 2 is correctly *not* added to
    /// `tainted_indices`; the read at index 2 of the original source
    /// is safe.
    #[test]
    fn end_to_end_mid_chain_inc_rc_on_chain_link_prevents_later_taint() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 3]):
                v2 = array_set v0, index u32 0, value u32 10
                v4 = array_set v2, index u32 1, value u32 20
                inc_rc v4
                v6 = array_set v4, index u32 2, value u32 30
                v8 = array_get v0, index u32 2 -> u32
                return v8
            }"#;
        assert_verifier_accepts_because(
            src,
            "inc_rc v4 clears derived; subsequent array_set v4, 2 is on fresh storage \
             and doesn't taint v0[2]",
        );
    }

    /// Poseidon2-style interleaved chain: each read happens *before* the
    /// chain link that writes its index, so `tainted_indices` only
    /// covers earlier-in-program writes when each read is checked. This
    /// pattern was the original motivation for the index filter and
    /// must stay green under the chain-aware version.
    ///
    /// **Where this lives in the wild.** The SSA shape below is distilled
    /// from `<impl Hasher for Poseidon2Hasher>::finish_ref` in
    /// `noir_stdlib/src/hash/poseidon2.nr` (~lines 22–26):
    ///
    /// ```text
    /// state[0] += self._state[i * RATE];
    /// state[1] += self._state[i * RATE + 1];
    /// state[2] += self._state[i * RATE + 2];
    /// ```
    ///
    /// After `mem2reg_brillig` each `state[i] += ...` becomes an
    /// `array_get state, i` immediately followed by an `array_set` that
    /// extends the chain, producing the interleaved
    /// read-then-chain-write shape this test pins down.
    ///
    /// End-to-end coverage comes from
    /// `collections::umap::test::test_no_duplicate_keys_after_deletion_and_insertion`
    /// in the stdlib tests (`cargo nextest run -p nargo_cli --test stdlib-tests`),
    /// which transitively hashes a value via `Poseidon2Hasher::finish_ref`
    /// and so exercises this SSA shape under `debug_assertions`.
    #[test]
    fn end_to_end_interleaved_chain_writes_and_reads_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 4]):
                v2 = array_get v0, index u32 0 -> u32
                v3 = array_set v0, index u32 0, value v2
                v5 = array_get v0, index u32 1 -> u32
                v6 = array_set v3, index u32 1, value v5
                v8 = array_get v0, index u32 2 -> u32
                v9 = array_set v6, index u32 2, value v8
                v11 = array_get v0, index u32 3 -> u32
                return v11
            }"#;
        assert_verifier_accepts_because(
            src,
            "Poseidon2-style: each read at idx i precedes the chain write at idx i, \
             so tainted_indices doesn't yet cover the read",
        );
    }

    /// Direct shape that the `MakeArray`-non-aliasing filter must NOT
    /// silence: the `array_set`'s *source* is itself a `make_array` result,
    /// and the same `make_array` is read forward via `array_get`. The
    /// source-self-preservation rule in [`Context::alias_set_for`] keeps
    /// the source in its own alias-set regardless of the filter, so the
    /// walk still finds the aliased read and flags.
    #[test]
    fn end_to_end_array_set_on_make_array_with_forward_read_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v1 = make_array [u32 1, u32 2, u32 3] : [u32; 3]
                v3 = array_set v1, index u32 0, value u32 99
                v5 = array_get v1, index u32 0 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// Aliased shape that the `MakeArray`-non-aliasing filter must NOT
    /// silence: a `make_array` is threaded forward into a block parameter
    /// via a *forward* edge (no loop), the parameter is the `array_set`'s
    /// source, and the original `make_array` value is read forward. The
    /// `make_array` represents the same one-time allocation that the
    /// `array_set` may mutate in place at runtime — so dropping it from
    /// the alias-set would lose the hazard.
    #[test]
    fn end_to_end_make_array_aliased_via_forward_block_param_with_forward_read_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v1 = make_array [u32 1, u32 2, u32 3] : [u32; 3]
                jmp b1(v1)
              b1(v2: [u32; 3]):
                v4 = array_set v2, index u32 0, value u32 99
                v6 = array_get v1, index u32 0 -> u32
                return v6
            }"#;
        assert_verifier_rejects(src);
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// loop body that mutates the loop-variable (an `array_set` whose
    /// source is the loop-header parameter) and then re-seeds the
    /// loop variable with a global on the back-edge — the user-source
    /// equivalent of `for _ { a[i] = …; a = G_A }`. The loop-header
    /// param's backward set pulls in both the function arg (forward
    /// edge into the header) AND the global (back-edge into the
    /// header), so a post-loop `array_get` on the loop param appears
    /// as an aliased read of the function arg's storage. At runtime
    /// the loop param at the loop exit is always the global (last
    /// back-edge binding), and the global has been `inc_rc`'d, so its
    /// `RC ≥ 2` from iter 1+ and the `array_set` never mutates it; iter
    /// 0's mutation hits the function arg's caller-side storage,
    /// which is no longer referenced after iter 0's back-edge re-bind.
    /// The back-edge-participant relaxation accepts the SSA: the global
    /// is a non-source alias with an `inc_rc` and is a back-edge arg.
    #[test]
    fn end_to_end_loop_reseeded_with_inc_rcd_global_is_accepted() {
        let src = r#"
            g0 = u32 10
            g1 = u32 20
            g2 = u32 30
            g3 = make_array [u32 10, u32 20, u32 30] : [u32; 3]

            brillig(inline) fn main f0 {
              b0(v4: [u32; 3]):
                jmp b1(u32 0, v4)
              b1(v8: u32, v19: [u32; 3]):
                v9 = lt v8, u32 3
                jmpif v9 then: b2(), else: b3()
              b2():
                v14 = array_set v19, index u32 1, value u32 40
                inc_rc g3
                v16 = unchecked_add v8, u32 1
                jmp b1(v16, g3)
              b3():
                v18 = array_get v19, index u32 1 -> u32
                return v18
            }"#;
        assert_verifier_accepts_because(
            src,
            "the back-edge re-seeds the loop variable with an inc_rc'd global, so the post-loop read sees the global's pristine storage — not the function arg's caller-side storage that iter 0's array_set may have mutated",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// callee receives an array argument, makes a *fresh* `make_array`
    /// copy that's threaded into the loop header on the forward edge,
    /// mutates the loop variable inside the body, and then re-seeds the
    /// loop variable with the *original* argument on the back-edge.
    /// The original argument has its `inc_rc` emitted in the loop body
    /// (right before the back-edge), so iter 1+ sees `RC ≥ 2` on it
    /// and the `array_set` allocates fresh. Iter 0's `array_set` mutates
    /// the fresh local copy, which is no longer referenced after the
    /// back-edge re-bind — so the apparent post-array_set read of the
    /// loop variable (the next iteration's `array_get` on the loop
    /// header param) is actually reading the original argument's
    /// pristine storage. The back-edge-participant relaxation in
    /// [`Context::some_inc_rc_precedes`] accepts this: the original arg
    /// is a non-source alias with an `inc_rc`, and it appears as a
    /// back-edge arg — a codegen signal that the frontend is
    /// deliberately managing the loop aliasing.
    #[test]
    fn end_to_end_loop_reseeded_with_inc_rcd_entry_param_is_accepted() {
        let src = r#"
            brillig(inline_never) fn func_1 f0 {
              b0(v0: [[u8; 3]; 4]):
                v5 = array_get v0, index u32 0 -> [u8; 3]
                inc_rc v5
                inc_rc v5
                v6 = make_array [v5, v5, v5, v5] : [[u8; 3]; 4]
                jmp b1(u32 0, v6)
              b1(v9: u32, v24: [[u8; 3]; 4]):
                v10 = lt v9, u32 3
                jmpif v10 then: b2(), else: b3()
              b2():
                v13 = array_get v24, index u32 3 -> [u8; 3]
                inc_rc v13
                v20 = make_array b"XDR"
                v25 = array_set v24, index u32 3, value v20
                inc_rc v0
                v23 = unchecked_add v9, u32 1
                jmp b1(v23, v0)
              b3():
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 (the function arg) is a back-edge arg with an inc_rc in the loop body, so iter 1+'s array_set on v0 allocates fresh; iter 0's array_set mutates v6 (the fresh forward-edge make_array), which is no longer referenced after the back-edge re-bind. The back-edge-participant relaxation accepts the inc_rc on v0 as the codegen signal.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern with
    /// *nested* loops: an inner-loop body mutates the inner-loop's
    /// header parameter, but the value that gets RC-protected by an
    /// `inc_rc` (the function arg `v0`) sits on the *outer* loop's
    /// back-edge — not directly on the inner loop's back-edge. The
    /// inner source's backward alias-set still contains `v0`
    /// transitively, via the chain `inner_param ← outer_param ← v0`
    /// (outer back-edge).
    ///
    /// The back-edge-participant relaxation in
    /// [`Context::some_inc_rc_precedes`] handles this uniformly: `v0`
    /// is a non-source alias with an `inc_rc` and appears as a back-edge
    /// arg (on the outer back-edge), so the `array_set` is accepted.
    #[test]
    fn end_to_end_nested_loop_outer_back_edge_inc_rcd_arg_filtered_for_inner_source() {
        let src = r#"
            brillig(inline) fn func_1 f0 {
              b0(v0: [u1; 3], v1: [u1; 3]):
                jmp b1(u32 0, v1)
              b1(v5: u32, v21: [u1; 3]):
                v7 = lt v5, u32 4
                jmpif v7 then: b2(), else: b3()
              b2():
                v11 = array_get v21, index u32 0 -> u1
                jmpif v11 then: b4(), else: b5()
              b3():
                return
              b4():
                jmp b6(v21)
              b5():
                jmp b7(u32 0, v21)
              b6(v23: [u1; 3]):
                inc_rc v0
                v20 = unchecked_add v5, u32 1
                jmp b1(v20, v0)
              b7(v14: u32, v22: [u1; 3]):
                v15 = lt v14, u32 2
                jmpif v15 then: b8(), else: b9()
              b8():
                v16 = array_get v0, index u32 0 -> u1
                v18 = array_set v22, index u32 0, value v16
                v19 = unchecked_add v14, u32 1
                jmp b7(v19, v18)
              b9():
                jmp b6(v22)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 is in the inner source's backward set transitively (via outer header v21's back-edge) and has an `inc_rc`. The back-edge-participant relaxation accepts: v0 ≠ source and v0 is a back-edge arg (on the outer back-edge), so the inc_rc on v0 is taken as the codegen signal that loop aliasing is being managed.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// function arg that's *both* forward-threaded into an unrelated
    /// early-return branch *and* back-edge-threaded into the outer loop
    /// that carries an inner-loop `array_set`. The `inc_rc` in the
    /// outer-loop body guarantees `RC ≥ 2` by the time the inner source
    /// actually equals the arg at runtime, so the `array_set` on it is
    /// forced to allocate fresh.
    #[test]
    fn end_to_end_inc_rcd_arg_forward_in_unrelated_branch_and_back_to_outer_loop_is_accepted() {
        let src = r#"
            brillig(inline) fn func_1 f0 {
              b0(v0: [u1; 3], v1: [u1; 3], v2: u1):
                jmpif v2 then: b1(), else: b2()
              b1():
                jmp b_join(v0, v1)
              b_join(v_a: [u1; 3], v_b: [u1; 3]):
                jmp b_exit()
              b_exit():
                return
              b2():
                jmp b_outer(u32 0, v1)
              b_outer(v5: u32, v_outer_param: [u1; 3]):
                jmpif u1 1 then: b_outer_body(), else: b_exit()
              b_outer_body():
                jmp b_inner(u32 0, v_outer_param)
              b_inner(v6: u32, v_inner_param: [u1; 3]):
                jmpif u1 1 then: b_inner_body(), else: b_outer_tail()
              b_inner_body():
                v_set = array_set v_inner_param, index u32 0, value u1 0
                v7 = unchecked_add v6, u32 1
                jmp b_inner(v7, v_set)
              b_outer_tail():
                inc_rc v0
                v8 = unchecked_add v5, u32 1
                jmp b_outer(v8, v0)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 is a back-edge arg to the outer loop (and also a forward arg to b_early_exit on an unrelated branch) carrying an inc_rc. The back-edge-participant relaxation accepts: v0 ≠ source, v0 ∈ back_edge_args, and has inc_rc — the codegen signal that the outer-loop iteration aliasing is being managed.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern:
    /// `..=` (inclusive range) generates an extra `array_set v_loop`
    /// in a post-loop block, which forward-threads the back-edge value
    /// (`v0`) into a downstream block param (`v25`). The walk reaches
    /// a `array_get v25` and would flag it because the backward set
    /// pulls `v25` into the source `v24`'s alias-set via the
    /// post-loop forward edge.
    ///
    /// At runtime `v25` is always `v0` (or only `v1` on a path where
    /// the `array_set` never fired), and the `inc_rc v0` inside the
    /// loop body ensures `RC(v0) ≥ 2` for every iteration where the
    /// `array_set` is on `v0`. The verifier doesn't reason path-
    /// sensitively, but `inc_rc v0` *is* on an alias (`v0 != v24`)
    /// that participates in the loop's back-edge — that's a deliberate
    /// codegen signal that aliasing is being managed. The
    /// back-edge-participant relaxation in `some_inc_rc_precedes`
    /// accepts on that basis.
    #[test]
    fn end_to_end_inc_rc_on_back_edge_participant_alias_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u1; 3], v1: [u1; 3]):
                jmp b1(i16 -9421, v1)
              b1(v5: i16, v24: [u1; 3]):
                v8 = lt v5, i16 -9417
                jmpif v8 then: b2(), else: b3()
              b2():
                v14 = array_set v24, index u32 0, value u1 0
                inc_rc v0
                v16 = unchecked_add v5, i16 1
                jmp b1(v16, v0)
              b3():
                jmpif u1 1 then: b4(), else: b5(v24)
              b4():
                v21 = array_set v24, index u32 0, value u1 0
                inc_rc v0
                jmp b5(v0)
              b5(v25: [u1; 3]):
                v23 = array_get v25, index u32 0 -> u1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "inc_rc v0 is on an alias of the source v24 (not v24 itself) that's also a back-edge arg to v24 via the b2->b1 back-edge — a codegen signal that loop aliasing is being managed",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: two
    /// array-typed function entry parameters both flow into the same
    /// downstream block parameter — one via a forward edge into the
    /// loop header, the other via a back-edge that re-seeds the loop
    /// variable with the second entry parameter (the user-source-level
    /// equivalent of `c = b` at the bottom of an outer loop iteration).
    /// The loop-header param's backward set therefore contains both
    /// entry params, and the walk would find an aliased `array_get` of
    /// the *other* entry param on a forward path from the `array_set`.
    /// The codegen emits `inc_rc v_b` inside the loop body (right
    /// before the back-edge), which is the signal that loop-iteration
    /// aliasing is being managed: `v_b` is a non-source alias and a
    /// back-edge arg, so the back-edge-participant relaxation accepts.
    #[test]
    fn end_to_end_two_entry_array_params_cross_aliased_via_back_edge_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v_b: [i8; 4], v_c: [i8; 4]):
                jmp b1(v_c)
              b1(v_loop: [i8; 4]):
                jmpif u1 1 then: b3(), else: b2()
              b2():
                return
              b3():
                v6 = array_get v_b, index u32 0 -> i8
                v7 = array_set v_loop, index u32 0, value i8 0
                inc_rc v_b
                jmp b1(v_b)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v_b and v_c both flow into the loop-header param's backward set, but at runtime they're distinct caller-side storages; the inc_rc v_b on the back-edge is the codegen signal the back-edge-participant relaxation accepts on",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern where
    /// the `inc_rc`'d value reaches the loop back-edge through a *latch*
    /// block rather than as the literal back-edge arg. Source-level shape
    /// (`func_1` from the minimized repro):
    ///
    /// ```ignore
    /// for _ in 0..2 {
    ///     c.0 = if a { c.0[0] = 40; b.3 } else { [50; 1] };
    /// }
    /// assert(b.3[0] == 10);
    /// ```
    ///
    /// The loop variable `v28` (`c.0`) is path-dependent: on the first
    /// iteration it is the forward seed `v6`, and the `array_set v28`
    /// mutates that (dead-after) storage; on later iterations it is `v4`
    /// (`b.3`), threaded back as `v4 → v23 → v28`. The frontend emits
    /// `inc_rc v4` in `b4` right before threading `v4` forward into the
    /// latch `b6`, so by the time `v4` re-enters as `v28` its `RC ≥ 2`
    /// and the `array_set` copies — the post-loop `array_get v4` reads
    /// pristine storage.
    ///
    /// The literal back-edge arg is `v23` (the latch param), not `v4`, so
    /// the back-edge-participant relaxation in
    /// [`Context::some_inc_rc_precedes`] does not fire. Acceptance comes
    /// from the protected-participant filter in
    /// [`Context::find_reachable_aliased_use`]: `v4` is a back-edge
    /// participant (it flows into `v23` through the forward `b4 → b6`
    /// edge) and carries its own `inc_rc`, so it is dropped from the
    /// use-set; the per-arg kill rule then drops `v28` along the
    /// back-edge and the `array_get v4` is never flagged.
    #[test]
    fn end_to_end_inc_rcd_value_reaches_back_edge_through_latch_is_accepted() {
        let src = r#"
            brillig(inline) fn func_1 f0 {
              b0(v0: u1, v4: [i32; 1], v6: [i32; 1]):
                jmp b1(u32 0, v6)
              b1(v14: u32, v28: [i32; 1]):
                v15 = lt v14, u32 2
                jmpif v15 then: b2(), else: b3()
              b2():
                jmpif v0 then: b4(), else: b5()
              b3():
                v25 = array_get v4, index u32 0 -> i32
                constrain v25 == i32 10, "HNJ"
                return
              b4():
                v20 = array_set v28, index u32 0, value i32 40
                inc_rc v4
                jmp b6(v4, v20)
              b5():
                v22 = make_array [i32 50] : [i32; 1]
                jmp b6(v22, v28)
              b6(v23: [i32; 1], v29: [i32; 1]):
                v24 = unchecked_add v14, u32 1
                jmp b1(v24, v23)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v4 is a back-edge participant (flows into the latch param v23 via the forward b4->b6 edge) with its own inc_rc, so the protected-participant filter drops it and the array_get v4 is never flagged",
        );
    }

    /// Soundness counterpart to
    /// [`end_to_end_inc_rcd_value_reaches_back_edge_through_latch_is_accepted`]:
    /// a latch `b6` joins two predecessors that feed the loop's back-edge
    /// position, one carrying an `inc_rc`'d value (`v0`) and the other an
    /// unprotected entry parameter (`v1`) with no `inc_rc`. On the path
    /// through `b5` the loop variable becomes `v1`, and `array_set v4`
    /// mutates `v1`'s storage in place (RC = 1); the post-loop
    /// `array_get v1` then observes that mutation — a genuine hazard.
    ///
    /// The protected-participant filter must **not** be fooled into
    /// exonerating `v1` because its *sibling* `v0` is protected: the
    /// filter is gated per value on that value's own `inc_rc`. `v1` has
    /// none, so it stays in the use-set and the walk flags the read.
    /// (Widening the relaxation to "some back-edge participant has an
    /// `inc_rc`" would wrongly accept this.)
    #[test]
    fn end_to_end_latch_join_with_unprotected_sibling_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [i32; 1], v1: [i32; 1], v2: u1):
                jmp b1(u32 0, v0)
              b1(v3: u32, v4: [i32; 1]):
                v5 = lt v3, u32 2
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = array_set v4, index u32 0, value i32 40
                jmpif v2 then: b4(), else: b5()
              b3():
                v7 = array_get v1, index u32 0 -> i32
                return
              b4():
                inc_rc v0
                jmp b6(v0)
              b5():
                jmp b6(v1)
              b6(v8: [i32; 1]):
                v9 = unchecked_add v3, u32 1
                jmp b1(v9, v8)
            }"#;
        assert_verifier_rejects(src);
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern (minimized):
    /// the protected back-edge participant is itself an inner-loop-header
    /// parameter that an in-use alias flows into, so the forward walk's
    /// add-rule would otherwise re-introduce it.
    ///
    /// `v78` (the inner-loop-header param read by the `call` in `b7`) is in
    /// `v85`'s backward alias-set — via `v85 <- v79 <- v91 <- v78`, where the
    /// `b7 -> b11` edge crosses `v78` into the source column (`v91`). It
    /// carries its own `inc_rc` and is a back-edge participant (it reaches
    /// the `b11 -> b6` back-edge arg `v91`). The `inc_rc` is therefore
    /// loop-carried: by the time `v78`'s storage comes around to be the
    /// `array_set v85` source it is RC >= 2, so the `array_set` copies and the
    /// `call`'s read is safe.
    ///
    /// The removal must be **sticky**: `v78` is dropped from the seed, but
    /// without excluding it from the add-rule too it is re-introduced the
    /// moment the in-use alias `v72` flows into it at `b6`, and the `call`
    /// is flagged. (`v72`, the second outer column, is the live carrier that
    /// survives the outer back-edge and re-feeds `v78`.)
    #[test]
    fn end_to_end_sticky_removal_of_inner_loop_header_participant_is_accepted() {
        let src = r#"
            brillig(inline) fn f f0 {
              b0(v0: u1, v72i: [u32; 1], v73i: [u32; 1]):
                jmp b1(v72i, v73i)
              b1(v72: [u32; 1], v73: [u32; 1]):
                jmpif v0 then: b6(v72, v73), else: b3()
              b3():
                return u1 0
              b6(v78: [u32; 1], v79: [u32; 1]):
                jmpif v0 then: b7(), else: b12(v78, v79)
              b7():
                inc_rc v78
                v51 = call f0(v0, v78, v79) -> u1
                jmp b11(v79, v78)
              b11(v90: [u32; 1], v91: [u32; 1]):
                jmp b6(v90, v91)
              b12(v84: [u32; 1], v85: [u32; 1]):
                v66 = array_set v85, index u32 0, value u32 9
                jmp b1(v84, v66)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v78 is an inner-loop-header param in v85's backward set with its own loop-carried inc_rc and back-edge participation; sticky removal keeps it out of the use-set so the add-rule can't re-introduce it for the call to read",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered minimal shape:
    /// two array-typed function entry parameters flow into the same
    /// downstream block parameter via a *forward* if-else sibling join
    /// (no loops, no `inc_rc` anywhere). Source-level shape:
    ///
    /// ```ignore
    /// fn main(a, mut b, c) -> Field {
    ///     if c == 0 { b[1] = b[0]; b[0] = 20; b = a; b[1] }
    ///     else { c }
    /// }
    /// ```
    ///
    /// Distinct entry parameters point at distinct caller-side storage
    /// by Brillig calling convention. The backward walk keeps them
    /// apart — from `v1`'s perspective (an entry param) its backward
    /// set is just `{v1}`, so `v0` never enters the alias-set and no
    /// flag fires on the cross-arg `array_get`.
    #[test]
    fn end_to_end_two_entry_array_params_cross_aliased_via_forward_sibling_join_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [Field; 2], v1: [Field; 2], v3: Field):
                v5 = eq v3, Field 0
                jmpif v5 then: b1(), else: b2()
              b1():
                v11 = array_get v1, index u32 0 -> Field
                v13 = array_set v1, index u32 1, value v11
                v16 = array_set v13, index u32 0, value Field 20
                v18 = array_get v0, index u32 1 -> Field
                jmp b3(v18, v0)
              b2():
                jmp b3(v3, v1)
              b3(v19: Field, v20: [Field; 2]):
                return v19
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 and v1 are both array-typed function entry params flowing into b3.v20 via the if-else sibling join. The backward walk from v1 (an entry param) doesn't pull v0 into the alias-set, so the cross-arg array_get isn't flagged.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: an
    /// `array_set` whose source flows through a forward (non-loop)
    /// sibling-join with a global on the other arm, and the codegen
    /// emits an `inc_rc` on the global right before the forward jump
    /// that threads the global into the join's block parameter.
    /// Source-level shape:
    ///
    /// ```ignore
    /// fn func_1(mut b: [[bool; 3]; 4]) {
    ///     if b[0][0] {
    ///         b[0] = b[3];
    ///         b = G_A;       // re-seed `b` with a global
    ///     }
    ///     b[0] = b[1];
    /// }
    /// ```
    ///
    /// [`Context::backward_aliases`] doesn't pull `g0` into `v0`'s
    /// alias-set (v0 is an entry param with no predecessors), so the
    /// alias-set stays `{v0}` and the walk never touches `v5` or `g0`.
    /// Kept as a precision regression: if the alias analysis ever
    /// loses precision on forward-edge sibling joins, this test will
    /// start failing.
    #[test]
    fn end_to_end_inc_rc_on_forward_edge_participant_alias_is_accepted() {
        let src = r#"
            g0 = make_array [u1 0, u1 0, u1 0, u1 0] : [u1; 4]

            brillig(inline) fn main f0 {
              b0(v0: [u1; 4]):
                v2 = array_get v0, index u32 0 -> u1
                jmpif v2 then: b1(), else: b2(v0)
              b1():
                v4 = array_set v0, index u32 0, value u1 1
                inc_rc g0
                jmp b2(g0)
              b2(v5: [u1; 4]):
                v7 = array_get v5, index u32 1 -> u1
                v9 = array_set v5, index u32 0, value u1 1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "backward_aliases doesn't pull g0 into v0's alias-set (v0 is an entry param with no predecessors), so the walk's add-on-edge rule sees arg g0 ∉ use_set at b1 → b2 and doesn't propagate v5 into use_set; no inc_rc relaxation needed",
        );
    }

    /// Counterpart of
    /// [`end_to_end_inc_rc_on_forward_edge_participant_alias_is_accepted`]
    /// with `inc_rc g0` stripped. The codegen threads the global `g0`
    /// from one branch and the function arg `v0` from the other into the
    /// join's block parameter `v5`. The `array_set v0` lives only on the
    /// branch that subsequently passes `g0` (not `v0`) to the join.
    ///
    /// # Why it's safe at runtime
    ///
    /// - **then-branch (b0 → b1 → b2):** `array_set v0` may mutate `v0`'s
    ///   storage in place. The forward jump rebinds `v5 := g0`, so the
    ///   downstream reads of `v5` hit `g0`'s storage, not the mutated
    ///   `v0`.
    /// - **else-branch (b0 → b2):** `v0` is never mutated. `v5 := v0`,
    ///   reads of `v5` see the original `v0`.
    ///
    /// On the path where the mutation happens, the value threaded
    /// forward into `v5` is *not* the one that got mutated.
    ///
    /// # Why the analysis accepts this
    ///
    /// The backward walk follows `param ← arg` edges directionally:
    /// from `v0`'s perspective (an entry param with no predecessors),
    /// the backward set is just `{v0}`. The forward walk's add-on-edge
    /// rule then watches for `v0` to be threaded into a downstream
    /// param — but the b1→b2 edge passes `g0`, not `v0`, so no param
    /// gets added. The walk terminates without flagging.
    #[test]
    fn end_to_end_forward_edge_sibling_join_without_inc_rc_is_accepted() {
        let src = r#"
            g0 = make_array [u1 0, u1 0, u1 0, u1 0] : [u1; 4]

            brillig(inline) fn main f0 {
              b0(v0: [u1; 4]):
                v2 = array_get v0, index u32 0 -> u1
                jmpif v2 then: b1(), else: b2(v0)
              b1():
                v4 = array_set v0, index u32 0, value u1 1
                jmp b2(g0)
              b2(v5: [u1; 4]):
                v7 = array_get v5, index u32 1 -> u1
                v9 = array_set v5, index u32 0, value u1 1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "backward-alias-set of v0 is {v0}; the b1→b2 edge passes g0 (not v0), so the add-on-edge rule doesn't pull v5 into the use-set; reads of v5 are unrelated to v0's storage",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: an
    /// `array_set v0` in one branch is followed in the same block by an
    /// `inc_rc w` of a *different* value `w` that the backward
    /// alias-set walk places in `v0`'s alias-set (because both `v0` and
    /// `w` flow into the same `b3` block parameter from two branches).
    /// The `inc_rc` is a ref-count bump, not a content read — the walk
    /// handles `IncrementRc` explicitly (it can only *clear* `derived`,
    /// never count as an aliased use). Symmetric to the
    /// `Instruction::ArraySet` / `Call` "non-aliasing-result" filter: an
    /// instruction whose semantics don't read pre-mutation storage is
    /// not a hazard.
    #[test]
    fn end_to_end_array_set_followed_by_inc_rc_of_aliased_param_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 1], v1: [u32; 1]):
                jmpif u1 1 then: b1(), else: b2()
              b1():
                jmp b3(v0)
              b2():
                v5 = array_set v0, index u32 0, value u32 99
                inc_rc v1
                jmp b3(v1)
              b3(v6: [u32; 1]):
                return v6
            }"#;
        assert_verifier_accepts_because(
            src,
            "`inc_rc v1` after the array_set is a ref-count op, not a read of array contents",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered shape: a
    /// `make_array` defined in the same block as (and *after*) the
    /// `array_set`, whose result feeds the loop-header parameter on the
    /// loop's back-edge. The `make_array` is iteration-local (back-edge
    /// arg), so the `iteration_local_fresh` filter drops it from
    /// the alias-set: the per-arg kill at the back-edge then sees the
    /// arg ∉ `use_set` and correctly drops the loop-header parameter, so
    /// the walk terminates without flagging the loop-body reads.
    #[test]
    fn end_to_end_loop_back_edge_with_post_array_set_make_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 4]):
                jmp b1(v0, u32 0)
              b1(v1: [u32; 4], v2: u32):
                jmpif u1 1 then: b3(), else: b2()
              b2():
                return
              b3():
                call f1(v1)
                v6 = array_set v1, index u32 3, value u32 0
                v7 = make_array [u32 0, u32 0, u32 0, u32 0] : [u32; 4]
                v9 = add v2, u32 1
                jmp b1(v7, v9)
            }
            brillig(inline) fn observer f1 {
              b0(v0: [u32; 4]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "v7 (make_array) is defined after the array_set in the same block — it can't represent the mutated storage, and the loop-header param v1 is bound to v7 on the back-edge",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered shape that
    /// exercises the post-array_set-in-same-block filter via a
    /// forward-then-back round-trip: a `make_array` defined *after* an
    /// `array_set` in the same block (`b13`) gets threaded forward to
    /// a different block (`b14`) and from there *back* to the
    /// `array_set`'s source's parameter (`v29`) via a loop back-edge
    /// `b8 → b4`. Without the filter, that round-trip would pull the
    /// `make_array` (`v24`) into the source's backward set, the per-arg
    /// kill on the `b13 → b14(v24, _)` edge would see `v24 ∈ use_set`
    /// and refuse to kill `v26`, the next `b8 → b4(v26)` would see
    /// `v26 ∈ use_set` and refuse to kill `v29`, and the walk would
    /// flag the loop-header's `array_get v29` as an aliased read —
    /// even though at runtime `v24` is fresh storage that didn't exist
    /// at the `array_set`'s program point and can't carry pre-mutation
    /// aliasing forward.
    ///
    /// Source-level shape:
    ///
    /// ```ignore
    /// fn func_3(mut a: [bool; 1]) {
    ///     for _ in 0..3 {
    ///         while (a[0]) {
    ///             loop {
    ///                 a = if a[0] {
    ///                     a[0] = if a[0] { !a[0] } else { a[0] };
    ///                     [a[0]]
    ///                 } else { a };
    ///                 break;
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// The post-array_set-in-same-block filter in
    /// [`Context::alias_set_for`] drops `v24` from `v29`'s alias-set up
    /// front, so the per-arg kill correctly fires on `b13 → b14` and
    /// the rest of the walk terminates without flagging.
    #[test]
    fn end_to_end_post_array_set_make_array_round_tripped_through_loop_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u1; 1]):
                jmp b1(u32 0, v0)
              b1(v4: u32, v28: [u1; 1]):
                v5 = lt v4, u32 3
                jmpif v5 then: b2(), else: b3()
              b2():
                jmp b4(v28)
              b3():
                return
              b4(v29: [u1; 1]):
                v9 = array_get v29, index u32 0 -> u1
                jmpif v9 then: b5(), else: b6()
              b5():
                jmp b7()
              b6():
                v27 = unchecked_add v4, u32 1
                jmp b1(v27, v29)
              b7():
                v11 = array_get v29, index u32 0 -> u1
                jmpif v11 then: b9(), else: b10()
              b8():
                jmp b4(v26)
              b9():
                v13 = array_get v29, index u32 0 -> u1
                jmpif v13 then: b11(), else: b12()
              b10():
                jmp b14(v29, v29)
              b11():
                v15 = array_get v29, index u32 0 -> u1
                v16 = not v15
                jmp b13(v16)
              b12():
                v18 = array_get v29, index u32 0 -> u1
                jmp b13(v18)
              b13(v19: u1):
                v21 = array_set v29, index u32 0, value v19
                v23 = array_get v21, index u32 0 -> u1
                v24 = make_array [v23] : [u1; 1]
                jmp b14(v24, v21)
              b14(v26: [u1; 1], v30: [u1; 1]):
                jmp b8()
            }"#;
        assert_verifier_accepts_because(
            src,
            "v24 (make_array) is defined in b13 after the array_set on v29; it can't carry v29's pre-mutation aliasing, even though the backward walk reaches it via the round-trip v24 → b14(v24,_) → v26 → b4(v26,_) → v29. The post-array_set-in-same-block filter drops v24, so the per-arg kill on b13 → b14 fires and the walk terminates without flagging.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered shape: a
    /// `make_array` defined in the outer-loop body *before* the inner
    /// loop's `array_set`, whose result is threaded back to the outer
    /// loop's header on the back-edge. The outer header's parameter
    /// would acquire the `make_array` via the backward chain (outer
    /// header → outer back-edge → `make_array`), pulling the `make_array`
    /// into the inner `array_set`'s alias-set. The per-arg kill on the
    /// outer back-edge would then see the `make_array` in the use-set
    /// and refuse to kill the outer header parameter, letting the
    /// walk reach an earlier-in-source `array_get` — even though at
    /// runtime the parameter is rebound to a fresh `make_array` on
    /// every back-edge crossing, so the iteration-aliasing is illusory.
    ///
    /// The `iteration_local_fresh` filter drops a `make_array`
    /// result that appears on a loop back-edge: the `make_array` always
    /// allocates fresh top-level storage, so it
    /// can't represent the pre-mutation storage of any `array_set` source.
    /// (Aliasing through *nested-array elements* of a `make_array` is a
    /// documented gap — see the module-level docs.)
    #[test]
    fn end_to_end_nested_loop_outer_back_edge_with_pre_array_set_make_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u64; 1]):
                jmp b1(v0, u32 0)
              b1(v22: [u64; 1], v23: u32):
                v5 = eq v23, u32 0
                jmpif v5 then: b3(), else: b4()
              b2():
                return
              b3():
                jmp b2()
              b4():
                v8 = add v23, u32 1
                v11 = array_get v22, index u32 0 -> u64
                v12 = make_array [v11] : [u64; 1]
                jmp b6(v22, u32 0)
              b5():
                jmp b1(v12, v8)
              b6(v24: [u64; 1], v25: u32):
                v16 = eq v25, u32 3
                jmpif v16 then: b8(), else: b9()
              b7():
                jmp b5()
              b8():
                jmp b7()
              b9():
                v18 = add v25, u32 1
                v21 = array_set v24, index u32 0, value u64 0
                jmp b10()
              b10():
                jmp b6(v21, v18)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v12 (make_array) on the outer back-edge to v22 is fresh storage by construction; the outer parameter v22 is rebound to v12 each outer iteration, so reading v22 in the next iteration's b4 observes the fresh array, not the inner array_set's mutated storage",
        );
    }

    /// End-to-end regression for the pattern in stdlib's `compute_root`
    /// (`array_dynamic_blackbox_input`). The loop body has *two* chained
    /// `array_set`s — the back-edge threads the second one's result back
    /// into the block-parameter, and the loop exit reads the parameter
    /// directly via `array_get`.
    ///
    /// Without the array_set-results filter, the alias-set would include
    /// the second `array_set`'s result; the per-arg kill rule at the
    /// back-edge would then see that result in the use-set and refuse to
    /// kill the parameter, letting the alias leak to the loop exit and
    /// producing a false positive on the post-loop `array_get`.
    #[test]
    fn end_to_end_loop_chained_array_sets_with_post_loop_read_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v8 = array_set v2, index u32 0, value u32 1
                v11 = array_set v8, index u32 1, value u32 2
                v13 = add v3, u32 1
                jmp b1(v11, v13)
              b3():
                v15 = array_get v2, index u32 0 -> u32
                return v15
            }"#;
        assert_verifier_accepts_because(
            src,
            "loop exit reads `v2`, which is rebound on the back-edge to the chained array_set's result; not a hazard",
        );
    }

    /// End-to-end: the poseidon-style "`array_set` then call returning a
    /// fresh array threaded back via the loop's back-edge". Reduced from
    /// `bench_2_to_17` (stdlib's `poseidon2::hash_internal`):
    ///
    /// - `v8 = array_set v2, idx 0, …` in the loop body.
    /// - `v9 = call permute(v8)` returns a *fresh* array.
    /// - Back-edge `jmp b1(v9, …)` threads the fresh call result back to
    ///   `v2` for the next iteration.
    /// - After the loop, `array_get v2, idx 0` reads the final loop state.
    ///
    /// Without the call-result filter, the backward walk would pull `v9`
    /// into `v2`'s alias-set via the back-edge, the per-arg kill at the
    /// back-edge would see `v9` still in the use-set, and the loop-exit
    /// `array_get v2` would be flagged as an aliased read. The
    /// call-result filter drops `v9` from the alias-set, so the per-arg
    /// kill fires and the walk doesn't reach the post-loop read.
    #[test]
    fn end_to_end_loop_array_set_then_call_returning_fresh_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v8 = array_set v2, index u32 0, value u32 99
                v9 = call f1(v8) -> [u32; 2]
                v11 = add v3, u32 1
                jmp b1(v9, v11)
              b3():
                v13 = array_get v2, index u32 0 -> u32
                return v13
            }
            brillig(inline) fn permute f1 {
              b0(v0: [u32; 2]):
                return v0
            }"#;
        assert_verifier_accepts_because(
            src,
            "loop exit reads `v2` but the back-edge threads a fresh call result, breaking the alias chain at the call boundary",
        );
    }

    /// End-to-end: `&mut [u32; 2]` parameter pattern (like stdlib's
    /// `quicksort::partition`). mem2reg can't eliminate the reference, so
    /// the loop body has `v_loaded = load v_ref` re-executed each iteration,
    /// followed by `array_set v_loaded; store`. The cycle re-enters the
    /// load's block, where the load is re-executed and produces a *fresh*
    /// runtime value — so the `array_get v_loaded` on the next iteration is
    /// not a hazard.
    ///
    /// This test verifies the re-execution kill rule: on entry to the
    /// load's defining block, the loaded value is dropped from the use-set.
    #[test]
    fn end_to_end_loop_load_array_set_store_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: &mut [u32; 2], v1: u32):
                jmp b1(u32 0)
              b1(v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = load v0 -> [u32; 2]
                v8 = array_get v6, index u32 0 -> u32
                v10 = array_set v6, index u32 0, value u32 99
                store v10 at v0
                v12 = add v3, u32 1
                jmp b1(v12)
              b3():
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "load result is re-executed each iteration; the cycle's array_get is not a hazard",
        );
    }

    /// The alias-equivalence classes deliberately do **not** unify an
    /// `array_set`'s source and result. This test establishes the design
    /// contract.
    ///
    /// In the program below, `inc_rc v0` forces `v2 = array_set v0` to copy,
    /// so `v2` is in fresh storage. The chain `v4 = array_set v2 ; v6 =
    /// array_set v4` then mutates `v2`'s storage in place at each step.
    /// A use of `v4` after `v6` (the `array_get v4` below) is a real hazard
    /// because no `inc_rc v4` protects it.
    ///
    /// `v2`, `v4`, `v6` are instruction results (not block parameters),
    /// so the backward walk has nothing to chase from `v6` — `v6`'s
    /// alias-set is just `{v6}`. The walk never reaches `v0` via the
    /// chain, and the absence of `inc_rc v4` correctly surfaces the
    /// violation. (Note: this test uses `last_array_set`, whose source
    /// is `v4`; the third `array_set`'s source is `v4`, whose alias-set
    /// is `{v4}` — see the assertion below.)
    #[test]
    fn alias_set_does_not_walk_array_set_chains() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                inc_rc v0
                v2 = array_set v0, index u32 0, value u32 1
                v4 = array_set v2, index u32 1, value u32 2
                v6 = array_set v4, index u32 0, value u32 3
                v8 = array_get v4, index u32 0 -> u32
                return v8
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);

        let ArraySetSite { source, alias_set, .. } =
            last_array_set(function, &ctx).expect("array_set present");

        // Only the source itself — no walking into the upstream chain links
        // (v2, v0) or any block-parameter predecessors.
        assert_eq!(alias_set.iter().copied().collect::<Vec<_>>(), vec![source]);
    }

    /// Two things at once:
    ///   1. The backward alias-set follows block-parameter edges — for
    ///      the `array_set` in `b5`, `v2`'s pre-header source (`v0`) lands
    ///      in the same set.
    ///   2. The fixed-point terminates on cycles. `b1`'s parameter `v2` has
    ///      two inbound jumps, including the back-edge from `b5` that
    ///      carries the `array_set`'s own result. The result is excluded
    ///      from the alias-set at lookup time so cycles through it don't
    ///      leak the post-mutation value into the live aliases.
    ///
    /// This is the well-formed program from the design discussion: the loop reads
    /// `v2` each iteration, mutates `v2` in place via the `array_set`, and threads
    /// the result back through the block-parameter. No `inc_rc` is needed because
    /// block-parameter threading already keeps the post-mutation value visible to
    /// the next iteration.
    #[test]
    fn alias_set_follows_block_params_and_terminates_on_cycles() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = array_get v2, index u32 0 -> u32
                v8 = eq v3, u32 1
                jmpif v8 then: b4(), else: b5()
              b3():
                return
              b4():
                v10 = eq v6, u32 99
                constrain v6 == u32 99
                jmp b5()
              b5():
                v11 = array_set v2, index u32 0, value u32 99
                v12 = add v3, u32 1
                jmp b1(v11, v12)
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);

        let ArraySetSite { source, alias_set, .. } =
            last_array_set(function, &ctx).expect("array_set present");

        // Expect `{v2, v0}`: the source itself plus the function's array
        // parameter that flows into `v2` via the pre-header jmp from `b0`.
        // The back-edge's argument is the array_set's own result and is
        // excluded by the array_set-result filter inside `alias_set_for`.
        let entry_v0 = function.dfg.block_parameters(function.entry_block())[0];
        assert_eq!(alias_set.len(), 2);
        assert!(alias_set.contains(&source));
        assert!(alias_set.contains(&entry_v0));
    }

    /// Minimal SSA pinning down the **unique necessity** of the
    /// [`Context::iteration_local_fresh`] filter. The other
    /// "value can't share storage at the `array_set`'s program point"
    /// filters — `non_aliasing_array_values` (ArraySet/Call results),
    /// `post-array_set-in-same-block`, and the walk's `def-block-entry`
    /// kill — *don't* cover this case.
    ///
    /// SSA shape:
    ///
    /// ```text
    /// b0(v0): jmp b1(v0)
    /// b1(v1):
    ///   v3 = array_get v1, 0        ← read of v1 reachable forward only via re-entry
    ///   v4 = make_array [...]       ← make_array in the HEADER (not array_set's block)
    ///   jmp b2
    /// b2:
    ///   v6 = array_set v1, 0, v3    ← array_set on v1
    ///   jmp b3
    /// b3:
    ///   jmp b1(v4)                  ← back-edge with v4 to b1
    /// ```
    ///
    /// **Why neither non-iteration_local filter would suffice:**
    /// - `non_aliasing_array_values`: only filters `ArraySet`/`Call`
    ///   results, not `MakeArray`s.
    /// - `post-array_set-in-same-block`: `v4`'s def-block is `b1`, not
    ///   the `array_set`'s block `b2`, so this filter never fires.
    /// - `def-block-entry kill` in [`Context::succ_use_set`]: fires
    ///   when *entering* the param's def-block. The walk reaches the
    ///   back-edge `b3 → b1` first, and the kill rule (rule 1) checks
    ///   `arg ∈ use_set` *before* rule 2 drops `v4`. With `v4` still
    ///   in `use_set`, the rule sees the arg as "still an alias" and
    ///   keeps `v1`. The subsequent `array_get v1` in `b1` then flags.
    ///
    /// `iteration_local_fresh` filters `v4` at alias-set
    /// construction time so it's never in the `use_set` in the first
    /// place. The kill rule on `b3 → b1` then correctly fires, `v1`
    /// is dropped, and the walk terminates without flagging.
    #[test]
    fn end_to_end_iteration_local_make_array_filter_uniquely_necessary() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 1]):
                jmp b1(v0)
              b1(v1: [u32; 1]):
                v3 = array_get v1, index u32 0 -> u32
                v4 = make_array [u32 0] : [u32; 1]
                jmp b2()
              b2():
                v6 = array_set v1, index u32 0, value v3
                jmp b3()
              b3():
                jmp b1(v4)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v4 is a make_array on the b3 → b1 back-edge — iteration-local fresh storage. The iteration_local_fresh filter drops it from v1's alias-set, enabling the per-arg kill at the back-edge to correctly fire on v1. Without this filter, the walk would falsely flag the b1.array_get v1 on re-entry.",
        );
    }

    /// Five `inc_rc` placements, each isolated on its own array parameter
    /// so the `inc_rcs` don't accidentally cover for each other. Tests the
    /// precedence check, which requires the `inc_rc` to be on **every** path
    /// to the `array_set` — either earlier in the same block, or in a block
    /// that **dominates** the `array_set`'s block:
    ///   - `v0`: same-block, `inc_rc` *earlier* than the `array_set` → **precedes**.
    ///   - `v1`: `inc_rc` in entry block (dominates everything) → **precedes**.
    ///   - `v2`: `inc_rc` in a sibling branch (b1) → does **not** precede.
    ///     b1 doesn't dominate the common-successor block b3 (the b2 path
    ///     skips the `inc_rc`), so the bump can't vouch for the `array_set`.
    ///   - `v3`: same-block, `inc_rc` *later* than the `array_set` → does
    ///     **not** precede (same-block comparison still requires earlier
    ///     position).
    ///   - `v4`: no `inc_rc` anywhere → does **not** precede.
    #[test]
    fn inc_rc_precedence_requires_dominating_position() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: [u32; 2], v2: [u32; 2], v3: [u32; 2], v4: [u32; 2], v5: u1):
                inc_rc v1
                jmpif v5 then: b1(), else: b2()
              b1():
                inc_rc v2
                jmp b3()
              b2():
                jmp b3()
              b3():
                inc_rc v0
                v8 = array_set v0, index u32 10, value u32 11
                v11 = array_set v1, index u32 20, value u32 21
                v14 = array_set v3, index u32 30, value u32 31
                inc_rc v3
                v17 = array_set v4, index u32 40, value u32 41
                v20 = array_set v2, index u32 50, value u32 51
                return v20
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);

        let entry_params = function.dfg.block_parameters(function.entry_block());
        let v0 = entry_params[0];
        let v1 = entry_params[1];
        let v2 = entry_params[2];
        let v3 = entry_params[3];
        let v4 = entry_params[4];

        let array_sets = find_array_sets(function, &ctx);
        assert_eq!(array_sets.len(), 5, "five array_set instructions expected");

        for ArraySetSite { block, idx, source, alias_set, .. } in &array_sets {
            let precedes = ctx.some_inc_rc_precedes(alias_set, *source, *block, *idx);
            let (expected, label) = if *source == v0 {
                (true, "v0: same-block earlier inc_rc")
            } else if *source == v1 {
                (true, "v1: entry-block inc_rc")
            } else if *source == v2 {
                (false, "v2: inc_rc in sibling branch does not dominate")
            } else if *source == v3 {
                (false, "v3: same-block later inc_rc")
            } else if *source == v4 {
                (false, "v4: no inc_rc")
            } else {
                panic!("unexpected array_set source {source:?}");
            };
            assert_eq!(precedes, expected, "{label}: expected precedes={expected}, got {precedes}");
        }
    }

    /// Well-formed example: in-loop `array_get v2` is fine because `v2`
    /// is a block parameter that is re-bound each iteration to the
    /// `array_set`'s result via the back-edge. The forward walk should kill
    /// `v2` from the use-set on entry to `b1`, and the `array_get v2` in `b2`
    /// (reached via cycle) is no longer aliased.
    #[test]
    fn reachable_use_walk_kills_block_param_on_entry() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = array_get v2, index u32 0 -> u32
                v8 = eq v3, u32 1
                jmpif v8 then: b4(), else: b5()
              b3():
                return
              b4():
                v10 = eq v6, u32 99
                constrain v6 == u32 99
                jmp b5()
              b5():
                v11 = array_set v2, index u32 0, value u32 99
                v12 = add v3, u32 1
                jmp b1(v11, v12)
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let ArraySetSite {
            block, idx, instruction_id, source, alias_set, write_index_const, ..
        } = first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(
                &alias_set,
                source,
                instruction_id,
                block,
                idx,
                write_index_const,
            )
            .is_some();
        assert!(
            !has_use,
            "well-formed loop: array_get v2 is reached via cycle but v2 is rebound at b1; no aliased use should be found"
        );
    }

    /// PR-12671 malformed repro: the in-loop `array_get v0` reads the
    /// pre-header value `v0` directly, which is in the alias-set of the
    /// `array_set`'s source `v2`. `v0` is *not* a parameter of any block on the
    /// cycle, so it stays live in the use-set, and the walk flags the read.
    #[test]
    fn reachable_use_walk_detects_unprotected_predecessor_read() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let ArraySetSite {
            block, idx, instruction_id, source, alias_set, write_index_const, ..
        } = first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(
                &alias_set,
                source,
                instruction_id,
                block,
                idx,
                write_index_const,
            )
            .is_some();
        assert!(
            has_use,
            "malformed loop: array_get v0 reads the pre-header value, which aliases the array_set's source via b1's pre-header jmp"
        );
    }

    /// Diamond-with-back-edges CFG: two predecessors of the `array_set`'s
    /// block (`b3`) each kill a *different* alias-set member (`b1` kills
    /// `v3`, `b2` kills `v4`). The forward walk re-enters `b3` via
    /// `b3 → b4 → b1 → b3` with use-set `{v4, v0, v1}` and via
    /// `b3 → b4 → b2 → b3` with `{v3, v0, v1}` — neither a subset of the
    /// other. The bookkeeping must record the **union** of explored
    /// use-sets at `b3` so the cycle terminates.
    ///
    /// No aliased read exists; the walk should return `false`. A bug in the
    /// merge logic would surface either as non-termination or as a false
    /// positive on the `array_set`'s own source `v5` (re-killed on each cycle
    /// entry).
    #[test]
    fn reachable_use_walk_merges_use_sets_across_paths() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: [u32; 2], v2: u1):
                jmpif v2 then: b1(v0), else: b2(v1)
              b1(v3: [u32; 2]):
                jmp b3(v3)
              b2(v4: [u32; 2]):
                jmp b3(v4)
              b3(v5: [u32; 2]):
                v8 = array_set v5, index u32 0, value u32 99
                jmpif v2 then: b4(), else: b5()
              b4():
                jmpif v2 then: b1(v5), else: b2(v5)
              b5():
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let ArraySetSite {
            block, idx, instruction_id, source, alias_set, write_index_const, ..
        } = first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(
                &alias_set,
                source,
                instruction_id,
                block,
                idx,
                write_index_const,
            )
            .is_some();
        assert!(
            !has_use,
            "no aliased read exists; the walk must terminate and return false despite re-entering b3 with non-overlapping use-sets"
        );
    }

    /// **Documents a known false negative.** When a single jmp passes the
    /// same value to multiple parameter positions (e.g. `jmp b1(v0, v0)`),
    /// the resulting sibling parameters refer to the same runtime storage,
    /// but the backward alias-set walk doesn't see this: it follows each
    /// param's predecessors directionally, so `v1`'s backward set is
    /// `{v1, v0}` and `v2`'s is `{v2, v0}` — neither contains the other.
    ///
    /// At runtime, `array_set v1` mutates `v0`'s storage in place if RC=1,
    /// and `array_get v2` would observe the mutation. The verifier should
    /// flag this, but it doesn't.
    ///
    /// Pinning the current behavior down: if a future change makes the
    /// analysis precise enough to catch this shape, flip the assertion
    /// to `assert_verifier_rejects`. See the module-level docs for
    /// where this gap sits in the precision/recall trade-off.
    #[test]
    fn end_to_end_sibling_args_to_same_value_is_false_negative() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0, v0)
              b1(v1: [u32; 2], v2: [u32; 2]):
                v5 = array_set v1, index u32 0, value u32 99
                v7 = array_get v2, index u32 0 -> u32
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "sibling-same-value blind spot: v1 and v2 share storage at runtime (both bound to v0 by b0's jmp), but backward-alias-set walks each param's predecessors directionally and doesn't co-mingle siblings",
        );
    }

    /// **Documents the same known false negative across a block boundary.**
    /// Same sibling pattern as
    /// [`end_to_end_sibling_args_to_same_value_is_false_negative`], but
    /// the aliased read is in a downstream block reached by threading
    /// the sibling param `v2` forward. The forward walk's add-on-edge
    /// rule does propagate `v2 → v7` correctly *once `v2` is in the
    /// use-set*, but `v2` never enters the use-set in the first place
    /// (it's not in `v1`'s backward alias-set), so the chain doesn't
    /// fire.
    #[test]
    fn end_to_end_sibling_args_across_jmp_is_false_negative() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0, v0)
              b1(v1: [u32; 2], v2: [u32; 2]):
                v5 = array_set v1, index u32 0, value u32 99
                jmp b3(v5, v2)
              b3(v6: [u32; 2], v7: [u32; 2]):
                v9 = array_get v7, index u32 0 -> u32
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "sibling-same-value blind spot extended across a forward jmp: v2 isn't in v1's backward alias-set, so the add-on-edge rule never pulls v7 into the use-set either",
        );
    }

    /// Counterpart of the above: when the jmp arg is the `array_set`'s own
    /// result (`v5`), the destination's parameter `v6` is rebound to a
    /// value that is *not* in the alias-set (the result was excluded at
    /// lookup time). The kill must fire so `v6` is removed from the
    /// use-set, and a downstream `array_get v6` must not be flagged.
    ///
    /// This is the "happy path" of the per-arg kill rule.
    #[test]
    fn reachable_use_walk_kills_param_rebound_to_array_set_result() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0)
              b1(v1: [u32; 2]):
                v4 = array_set v1, index u32 0, value u32 99
                jmp b2(v4)
              b2(v5: [u32; 2]):
                v7 = array_get v5, index u32 0 -> u32
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let ArraySetSite {
            block, idx, instruction_id, source, alias_set, write_index_const, ..
        } = first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(
                &alias_set,
                source,
                instruction_id,
                block,
                idx,
                write_index_const,
            )
            .is_some();
        assert!(
            !has_use,
            "b2's v5 is rebound to v4 (the array_set's result, excluded from alias-set), so it is killed and array_get v5 is not aliased"
        );
    }

    /// The `array_set` mutates the loop-invariant function parameter `v0`
    /// **directly** and discards its result (`v6` is unused); the aliased
    /// `array_get v0` (b2) is reachable from the `array_set` (b5) only by
    /// crossing the loop back-edge `b5 → b1 → b2` into the next iteration.
    /// The alias-set is therefore the singleton `{v0}`, and the hazard is
    /// surfaced purely by the forward walk returning to a read of the same
    /// un-threaded value — no block-parameter threading is involved, unlike
    /// `end_to_end_pr_12671_repro_is_rejected`, whose `array_set` is on a
    /// loop-carried copy and whose result is threaded back.
    ///
    /// The write index is dynamic (`v1`), so `tainted == None` and every
    /// aliased read is flagged. This is the loop/back-edge analogue of the
    /// straight-line `end_to_end_array_set_dynamic_index_with_array_get_is_rejected`.
    #[test]
    fn end_to_end_array_set_dynamic_index_across_back_edge_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(u32 0)
              b1(v1: u32):
                v2 = lt v1, u32 2
                jmpif v2 then: b2(), else: b3()
              b2():
                v3 = array_get v0, index u32 0 -> u32
                v4 = eq v1, u32 1
                jmpif v4 then: b4(), else: b5()
              b4():
                v5 = array_get v0, index u32 1 -> u32
                constrain v3 == v5, "iter 1 v0[0] should equal v0[1]=99 after mutation"
                jmp b5()
              b5():
                v6 = array_set v0, index v1, value u32 99
                v7 = unchecked_add v1, u32 1
                jmp b1(v7)
              b3():
                return
            }"#;
        assert_verifier_rejects(src);
    }

    /// Same direct-on-parameter, result-discarded loop shape as
    /// `end_to_end_array_set_dynamic_index_across_back_edge_is_rejected`.
    /// Here the `array_set v0` (b5) writes a **constant** index `0` and the
    /// back-edge-reachable `array_get v0` (b2) reads that same index `0`.
    /// The read's index is covered by the mutation's `tainted` set, so it
    /// is flagged. This is the loop/back-edge analogue of the straight-line
    /// `end_to_end_array_set_array_get_matching_constant_indices_is_rejected`,
    /// confirming the index-aware `tainted`-set check fires across a loop
    /// back-edge and not only within a single block.
    #[test]
    fn end_to_end_array_set_matching_constant_index_across_back_edge_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(u32 0)
              b1(v1: u32):
                v2 = lt v1, u32 2
                jmpif v2 then: b2(), else: b3()
              b2():
                v3 = array_get v0, index u32 0 -> u32
                v4 = eq v1, u32 1
                jmpif v4 then: b4(), else: b5()
              b4():
                constrain v3 == u32 99, "iter 1 v0[0] should be 99 after iter 0 mutated it in-place"
                jmp b5()
              b5():
                v6 = array_set v0, index u32 0, value u32 99
                v7 = unchecked_add v1, u32 1
                jmp b1(v7)
              b3():
                return
            }"#;
        assert_verifier_rejects(src);
    }

    /// A located `array_set` plus everything the per-array_set verifier
    /// checks would need. Returned by [`find_array_sets`] / [`first_array_set`]
    /// / [`last_array_set`] so each test reads one struct rather than a
    /// six-element tuple.
    struct ArraySetSite {
        block: BasicBlockId,
        idx: usize,
        instruction_id: InstructionId,
        source: ValueId,
        alias_set: im::HashSet<ValueId>,
        /// The `array_set`'s index when it is a numeric constant. `None`
        /// indicates a dynamic index, in which case the verifier
        /// conservatively flags every aliased use.
        write_index_const: Option<FieldElement>,
    }

    fn find_array_sets(function: &Function, ctx: &Context<'_>) -> Vec<ArraySetSite> {
        let mut out = Vec::new();
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, index, .. } = function.dfg[*instruction_id] {
                    out.push(ArraySetSite {
                        block: block_id,
                        idx,
                        instruction_id: *instruction_id,
                        source: array,
                        alias_set: ctx.alias_set_for(array, block_id, idx),
                        write_index_const: function.dfg.get_numeric_constant(index),
                    });
                }
            }
        }
        out
    }

    fn first_array_set(function: &Function, ctx: &Context<'_>) -> Option<ArraySetSite> {
        find_array_sets(function, ctx).into_iter().next()
    }

    fn last_array_set(function: &Function, ctx: &Context<'_>) -> Option<ArraySetSite> {
        find_array_sets(function, ctx).into_iter().last()
    }
}
