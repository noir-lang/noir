# Lints and warning control

A **lint** is an opinionated warning — a diagnostic that flags legitimate but
usually-undesirable code (an unused item, a return value that is constant, …). An author
who intends the flagged code silences the lint by naming its slug in an `#[allow(...)]`
attribute on the enclosing item. This follows Rust's lint-level attribute model; see the
[rustc lint levels reference](https://doc.rust-lang.org/rustc/lints/levels.html) for the
behaviour Noir mirrors. Only item-level `#[allow]` is supported today; `#[warn]` /
`#[deny]` / `#[expect]` and finer-grained placement are future work (see *Intended
direction*).

## Silencing backend warnings with `#[allow(...)]`

Most warnings are emitted by the frontend, where the attributes of the item being checked
are in scope, so `#[allow(...)]` is honored where the warning is raised. A few warnings are
only discovered by the backend — for example `constant_return` ("Return variable contains a
constant value"), raised during ACIR generation by `Context::convert_ssa_return` in
[the ACIR backend](../compiler/noirc_evaluator/src/acir/mod.rs) when an entry point's return
value turns out to be constant. At that stage the source attributes are no longer in scope.

For these warnings the attribute is resolved in the frontend and carried through the
pipeline as a per-function flag:

- monomorphization reads the attribute from the interner and stores `allow_constant_return`
  on the monomorphized [`Function`](../compiler/noirc_frontend/src/monomorphization/ast.rs);
- SSA generation copies it onto the function's
  [`DataFlowGraph`](../compiler/noirc_evaluator/src/ssa/ir/dfg.rs), alongside other
  per-function metadata such as the runtime;
- the emission site (`Context::convert_ssa_return` in
  [the ACIR backend](../compiler/noirc_evaluator/src/acir/mod.rs)) checks the flag on the
  function it is converting and does not construct the warning when it is set.

Resolving the level in the frontend and honoring it at the emission site is preferred over
emitting every warning and filtering the results afterwards. A per-function flag keeps the
decision attached to the function's identity, which is exact under inlining: an inlined call
stack can point into a different function than the one whose return is reported, so
filtering by source location afterwards cannot reliably attribute a warning to the function
the attribute was written on.

## Behaviour

`#[allow(constant_return)]` on a function suppresses the `constant_return` warning for that
function, and only that function. A reader can check the implementation against these
statements:

- **Scope is the function, not a lexical span.** Each ACIR entry point is handled on its
  own. `main` and every `#[fold]` function is a separate entry point that warns on a
  constant return of its own and is silenced only by an `#[allow]` on itself. `#[allow]` on
  `main` does not silence a `#[fold]` function, and an `#[allow]` on a `#[fold]` function
  does not silence `main`.
- **Inlined code is attributed to the surviving entry point.** When a constant flows out of
  an inlined helper and becomes `main`'s return value, the warning belongs to `main`. The
  attribute silences it when placed on `main`, not on the inlined helper.
- **A silenced warning is never constructed,** so it is absent from every artifact. Each
  consumer of `CompiledProgram.warnings` — `nargo`, the wasm bindings, any other tool —
  observes the same set. Because a silenced warning does not exist, `--deny-warnings` has
  nothing to promote to an error: an item-level `#[allow]` wins over `--deny-warnings`. This
  is distinct from `--silence-warnings`, a per-invocation flag applied when reporting, which
  hides warnings that were still produced.

## Intended direction

Backend warning control is one instance of a general lint framework: a lint registry,
`#[warn]` / `#[deny]` / `#[expect]` levels (following the
[rustc lint levels](https://doc.rust-lang.org/rustc/lints/levels.html) model), validation
of unknown lint names, and attachment sites finer than a whole item. It follows the same
principle — resolve lint levels in the frontend and honor them at the emission site — and is
tracked in [noir-lang/noir#7460](https://github.com/noir-lang/noir/issues/7460).
