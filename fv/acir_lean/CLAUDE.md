# Working in `fv/acir_lean`

Read `README.md` first: it explains what is proved and how the proofs stay
attached to the Rust.

## Reviewed and unreviewed code

The table in `README.md` ("What you must review") is the list of reviewed
files: the spec in `AcirLean/Spec/`, the entry points, the enforcement
scripts, the regeneration script, the CI workflows, and the Rust halves of the
pins (`fv_templates.rs`, `fv_semantics.rs`). They state what is claimed and how
it is enforced. Lean cannot tell whether they match intent, so keep them small
and plain, and add any new file of that kind to the table.
`AcirLean/Proofs/` is checked by Lean and `AcirLean/Templates/` is compared
byte-for-byte with the compiler's output; neither needs human review.

## Keep `REVIEWING.md` in sync

`REVIEWING.md` explains every reviewed Lean definition to a reader who does
not know Lean. Any change to `AcirLean/Spec/` or `Check.lean` updates it in the
same PR:

- quote changed definitions verbatim, and rewrite their plain-English
  explanation and the "Check:" note for a reviewer;
- mention every new definition, and drop removed ones;
- keep the glossary (Part 0) covering any new Lean syntax the files use.

`scripts/check_reviewing.py`, run by `scripts/check.sh`, fails when a quoted
line no longer appears in the reviewed files or a reviewed definition is never
mentioned. It cannot tell whether an explanation is still accurate, so reread
the explanation of anything you change.

## The SSA meaning must match Noir

`AcirLean/Spec/SsaSemantics.lean` says what each SSA instruction does. It must
agree with Noir's SSA interpreter
(`compiler/noirc_evaluator/src/ssa/interpreter/`) for ACIR functions. It may
fail where the interpreter succeeds, which only makes a program unprovable, but
it must never compute a different value or succeed where the interpreter fails.
State any such difference in the definition's doc comment and in
`REVIEWING.md`.

`cargo test -p noirc_evaluator --lib fv_semantics` checks this on a fixed grid
of values: it replays `ssa_semantics.golden`, written by `EmitSemantics.lean`,
through the interpreter. After changing `Instruction.run`, regenerate the file
with `lake env lean --run EmitSemantics.lean ssa_semantics.golden` and run that
test. When the test reports a difference, fix the spec (and the proofs), not the
golden file. When the spec gains a new instruction or type, add it to the grid.

## Commands

- `just fv-check`: build the proofs and run every check of the `FV Lean` job.
- `just fv-regen-corpus`: rebuild the corpus programs
  (`AcirLean/Templates/Corpus.lean`) from `fv_templates.rs`'s output, when
  `integer_gadgets_match_lean_templates` reports a changed `corpus` section.
Never use `sorry`, `axiom`, `native_decide` or other escape hatches;
`scripts/check.sh` rejects them.
