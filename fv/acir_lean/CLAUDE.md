# Working in `fv/acir_lean`

Read `README.md` first: it explains what is proved and how the proofs stay
attached to the Rust.

## Reviewed and unreviewed code

- `AcirLean/Spec/`, `Check.lean`, the `Emit*.lean` entry points,
  `scripts/check.sh` and the CI workflows are **reviewed**: they state what is
  claimed and how it is enforced. Lean cannot tell whether they match intent,
  so keep them small and plain.
- `AcirLean/Proofs/` is checked by Lean and `AcirLean/Templates/` is compared
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

`AcirLean/Spec/Programs2.lean` says what each SSA instruction does. It must
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
- `just fv-regen`: rebuild the proved test programs from the current compiler,
  when the `FV test programs` job reports that one of their circuits changed.
- `just fv-regen-all`: also bring in test programs that are not proved yet,
  adding them to `testProgramNames` (`AcirLean/Spec/Coverage.lean`). A
  program the checker rejects makes `fv-check` fail until it is listed, with
  the reason, in `uncoveredPrograms` (`AcirLean/Spec/Claims.lean`).

After changing the checker (`AcirLean/Proofs/Checker2.lean`) or the test-program
data, regenerate the certificates with
`lake env lean --run scripts/emit_certs.lean AcirLean/Proofs/TestProgramCerts.lean`
(`just fv-regen` does it too). `check.sh` fails while they are stale.

When `just fv-regen` fails because a covered program no longer compiles or no
longer fits the subset, do not work around it: either extend the checker, or
remove the name from `testProgramNames` and say why in the PR.

Never use `sorry`, `axiom`, `native_decide` or other escape hatches;
`scripts/check.sh` rejects them.
