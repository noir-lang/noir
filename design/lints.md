# Lints and warning control

## Silencing backend warnings with `#[allow(...)]`

Most warnings are emitted by the frontend, where the attributes of the item being
checked are directly available, so `#[allow(...)]` can be honored at the emission
site. Some warnings, however, are only discovered by the backend — for example
`constant_return` ("Return variable contains a constant value"), which is detected
during ACIR generation when an entry point's return value turns out to be constant.
At that stage the source attributes are no longer in scope.

For these warnings the attribute is resolved in the frontend and carried through the
pipeline as a per-function flag:

- monomorphization reads the attribute from the interner and stores it on the
  monomorphized `ast::Function` (e.g. `allow_constant_return`);
- SSA generation copies it onto the SSA function's `DataFlowGraph`, alongside other
  per-function metadata such as the runtime;
- the emission site (ACIR generation for `constant_return`) checks the flag on the
  function it is converting and does not construct the warning when it is set.

Resolving the level in the frontend and honoring it at the emission site is preferred
over emitting every warning and filtering the results afterwards (for example by
matching a warning's call stack against the body spans of annotated functions). A
per-function flag keeps lint policy in one place, and it is exact under inlining: an
inlined call stack can point into a different function than the one whose return is
being reported, so matching by source location cannot reliably attribute a warning to
the function the attribute was written on.

Consequences of the flag-based design:

- The warning is never constructed, so it never reaches any artifact. Every consumer
  (`nargo`, the wasm bindings, anything reading `CompiledProgram.warnings`) observes
  the same silenced set, and `--deny-warnings` has nothing to deny — an item-level
  `#[allow]` wins over the global flag by construction.
- The flag is scoped to the annotated function's identity, not its source span. Each
  ACIR entry point (`main` and every `#[fold]` function) warns and is silenced
  independently.
- The flag is not part of the SSA text syntax; hand-written SSA in tests leaves it
  unset. It is preserved wherever a pass rebuilds a function from an existing one
  (`Function::clone_signature`, `FunctionBuilder::from_existing`).

The broader lint-control framework — a lint registry, `#[warn]` / `#[deny]` /
`#[expect]` levels, and validation of unknown lint names — is tracked in
[noir-lang/noir#7460](https://github.com/noir-lang/noir/issues/7460). It follows the
same principle: resolve lint levels in the frontend and honor them at the emission
site.
