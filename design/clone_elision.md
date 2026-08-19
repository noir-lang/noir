# Clone elision for call arguments in Brillig

The ownership pass (`noirc_frontend/src/ownership/`) keeps Brillig's
copy-on-write array semantics honest by wrapping a value use in
`Expression::Clone` — lowered to an `inc_rc` in SSA — whenever that use is not
the value's last use. For call arguments this is often wasted work: a callee
that cannot modify its arguments needs no defensive reference count bump. Clone
elision is the optimization that skips those clones.

## Where the decision lives

The elision is decided **entirely in the ownership pass**
(`ownership/clone_elision.rs`): a `Clone` that survives to the monomorphized
AST is always honored by SSA generation. It was originally the other way
around — the ownership pass inserted clones unconditionally and `ssa_gen`'s
`codegen_call` peeled them off again — because the classification of builtins
lives in `noirc_evaluator`, which `noirc_frontend` cannot depend on.

That split caused a miscompilation. The deleting side reasoned only about the
callee ("can it modify its arguments?") while the hazard the clone guards
against can be created by the *caller*, between materializing one argument and
issuing the call. With the decision in the ownership pass, the code that knows
why the clone exists is also the code that decides whether it can be skipped.

## The rules

The `Clone` around argument `i` of a call is elided only when both hold:

1. **The callee preserves its arguments.** It neither modifies them nor
   returns an alias of them. Qualifying callees: builtins/low-level functions
   on an allowlist (see below), oracles (foreign calls copy their inputs
   across the runtime boundary), and "thin oracle wrappers" — functions whose
   body is exactly one forwarding call to an oracle or another wrapper, with
   structurally side-effect-free arguments.

2. **No later sibling argument can write.** Arguments are evaluated left to
   right, so everything in arguments `i+1..` runs after argument `i` has been
   materialized and before the call executes. If a later argument's evaluation
   can mutate caller-visible state (e.g. `bump(&mut x, i)` writing into the
   buffer argument `i` names), the elided `inc_rc` is exactly what would have
   made that write copy instead of landing in place — so the clone is kept
   unless every later argument is structurally side-effect-free. Condition 1
   alone is not sufficient: it describes the callee, while this hazard lives
   in the caller's argument-evaluation interval, which no callee-side
   predicate can describe.

Both conditions are conservative, syntactic checks; anything unrecognized
keeps the clone, which is always sound (at worst a wasted copy).

## The duplicated builtin classification

`builtin_supports_clone_elision` in the frontend duplicates information that
canonically lives in `noirc_evaluator` (`Intrinsic::purity` and
`Intrinsic::unsafe_for_clone_elision_in_brillig`), because the crate
dependency only goes the other way. Both sides are keyed on the shared
`noirc_frontend::shared::Builtin` enum, and the test
`ownership_clone_elision_list_matches_intrinsic_purity` in `noirc_evaluator`'s
`ssa_gen::tests` iterates every `Builtin` variant and asserts the two
classifications agree, so they cannot drift silently.

Notable entries: vector mutators are excluded because they may write through
their input pointer in place at reference count 1; `str_as_bytes` and
`array_as_str_unchecked` are excluded because their result aliases their
input, so a later mutation of the result would corrupt the source.
