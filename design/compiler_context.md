# Compiler context

How the state of a compilation is held and handed between the frontend, the driver and the
tools built on them (`nargo`, the language server, the wasm bindings).

## Two types, one per phase

`noirc_frontend::hir::Context` is the state while a compilation is being set up: the source
files, the crate graph, and the (still empty) results of analysis. Tools build it up through
`&mut Context`.

`Context::check_crate` consumes it, runs definition collection and elaboration, and returns a
`CheckedContext`. Everything which only makes sense on an elaborated crate takes a
`&CheckedContext`: finding `main`, the test functions, the fuzzing harnesses and the exported
functions of a crate, generating an ABI, and `noirc_driver::compile_no_check`.

The split exists so that calling one of those before checking the crate is a type error rather
than a panic, and so that a tool which keeps a checked context around (`nargo test` compiles
every test of a package against one) holds a value which nothing can reconfigure between uses.

### A checked context is returned even when checking fails

`check_crate` returns the `CheckedContext` next to the diagnostics, not inside the `Ok` of a
`Result`. The language server works on programs which do not compile and needs their analysis,
and every tool needs the file manager to report the errors.

### What `&CheckedContext` does and does not promise

`CheckedContext` dereferences to `&Context` and never to `&mut Context`. That rules out
reassigning its parts. It does not make the analysis immutable: the `NodeInterner` binds type
variables through shared cells. That monomorphizing one function leaves the interner fit for
monomorphizing the next is a property of the monomorphizer, covered by
`noirc_frontend::monomorphization::context_reuse_tests`.

`CheckedContext::interpret_function` runs comptime code, which can define and instantiate new
items, so it takes `&mut self`. It is the only method which does.

### Leaving the checked phase

`CheckedContext::into_context` gives the `Context` back. The language server uses it to take the
analysis apart into its per-package cache, which a single-file change then updates in place by
elaborating that file against it directly.

## Source files are shared, not borrowed

A `Context` holds its `FileManager` and parsed files as `Arc`s. They are read-only once a
context exists, and one set of them serves every package of a workspace, so each package's
context shares them. Holding them by reference instead would give `Context` lifetime
parameters, which every type that stores a context would then have to carry, and which
`wasm_bindgen` types cannot.

## Per-run settings are arguments

Settings which belong to one operation, and not to the program being analysed, are passed to
that operation instead of being stored on the context:

- `ComptimeIo`: where comptime code prints and whether its evaluations are tracked for
  coverage. Passed to the operations which run comptime code.
- `noirc_driver::BuildSettings`: where emitted artifacts go and which instrumentation is
  compiled in. Passed to the operations which generate code.

A context can therefore be reused for a second operation without first undoing what the
previous one configured.
