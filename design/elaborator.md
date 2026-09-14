# Per-item elaborator state

The elaborator's state splits in two. State that belongs to the whole elaboration (the
interner, the collected errors, the recursion depth, pending items) lives on `Elaborator`.
State that describes *the item being elaborated* lives in
[`ItemContext`](../compiler/noirc_frontend/src/elaborator/item_context.rs): the item's
module, its dependency id, `Self`, the trait and impl it belongs to, its generics and trait
bounds, the lambda, loop and `unsafe` block it is currently inside, whether it is comptime,
the module its visibility checks are made from, and the small counters used while elaborating
its body.

Elaborating an item can require elaborating another item first: a function body that calls a
function returning `impl Trait` needs the callee's body to learn the concrete type; a body
that mentions a global whose initializer is still pending elaborates that global; a body that
constructs, matches on or accesses a field of a type whose fields or variants are still pending
resolves them; `Expr::resolve` in comptime code resolves an expression under another function's
scope, which can in turn reach any of the above. The rule is:

- An entry point that can run in the middle of another item installs a **whole**
  `ItemContext` for the item it elaborates, through `Elaborator::with_item_context`, and the
  previous context is reinstated when it returns. It never swaps a hand-picked subset of
  fields, so a field added to `ItemContext` is saved and restored by construction.
- The item elaborated on demand starts from its own context, not the caller's: `Self`,
  generics, trait bounds, loops, `unsafe` blocks, comptime-ness and the caller's visibility
  module are not visible to it, and nothing it does to its context reaches the caller.
- Scoping *within* an item (a nested `unsafe` block, a loop body, a call's arguments, a type
  position where `impl Trait` is not allowed) uses enter/exit pairs on `ItemContext`, since
  those legitimately see and update the enclosing item's state.

The observable consequence is that an item is elaborated the same way whether it is reached in
declaration order or on demand from another item. The tests named
`lazily_elaborated_*_does_not_inherit_*` and `lazily_resolved_*_do_not_inherit_*` in
[`tests/traits/trait_as_type.rs`](../compiler/noirc_frontend/src/tests/traits/trait_as_type.rs),
[`tests/globals.rs`](../compiler/noirc_frontend/src/tests/globals.rs),
[`tests/structs.rs`](../compiler/noirc_frontend/src/tests/structs.rs),
[`tests/enums.rs`](../compiler/noirc_frontend/src/tests/enums.rs) and
[`tests/metaprogramming.rs`](../compiler/noirc_frontend/src/tests/metaprogramming.rs) each
pin one field of the context: a program that is rejected (or accepted) when the item is
elaborated in order must be rejected (or accepted) the same way when it is elaborated on
demand.

State that is set once on a fresh elaborator created for a comptime call, such as the names of
the runtime variables of the enclosing function used for diagnostics, stays on `Elaborator`:
it describes that elaborator's parent frame rather than any one item.
