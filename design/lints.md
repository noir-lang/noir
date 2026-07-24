# Lints and warning control

## Silencing backend warnings with `#[allow(...)]`

Most warnings are emitted by the frontend, where the attributes of the item being
checked are directly available, so `#[allow(...)]` can be honored at the emission
site. Some warnings, however, are only discovered by the backend — for example
`constant_return` ("Return variable contains a constant value"), which is detected
during ACIR generation when an entry point's return value turns out to be constant.
By that point source attributes are long gone.

For these warnings, the attribute is resolved in the frontend and carried through
the pipeline as a flag on the function:

- monomorphization reads the attribute from the interner and stores it on the
  monomorphized `ast::Function` (e.g. `allow_constant_return`);
- SSA generation copies it onto the SSA function's `DataFlowGraph`, next to other
  per-function metadata such as the runtime;
- the emission site (ACIR generation for `constant_return`) checks the flag on the
  function it is converting and simply never constructs the warning.

The alternative — emitting the warning unconditionally and filtering it out in
`noirc_driver` by matching the warning's call stack against the body spans of
annotated functions — was rejected (see PR #13399 review). Filtering after the fact
spreads lint policy across the pipeline, and matching by source location is fragile
under inlining: a call stack can point into a different function than the one whose
return is being reported.

Consequences of the flag-based design:

- The warning never exists in any artifact, so every consumer (`nargo`, the wasm
  bindings, anything reading `CompiledProgram.warnings`) observes the same silenced
  set, and `--deny-warnings` has nothing to deny. Item-level `#[allow]` therefore
  wins over the global flag by construction.
- The attribute is scoped to the annotated function's identity, not its source
  span. Each ACIR entry point (`main` and every `#[fold]` function) warns and is
  silenced independently.
- The flag is deliberately **not** part of the SSA text syntax: hand-written SSA in
  tests always has it unset. It must, however, be preserved wherever a pass rebuilds
  a function from an existing one (`Function::clone_signature`,
  `FunctionBuilder::from_existing`).

The general lint framework — a lint registry, `#[warn]`/`#[deny]`/`#[expect]`
levels, validation of unknown lint names — is tracked in issue #7460 and is expected
to build on the same principle: resolve lint levels in the frontend, honor them at
the emission site.
