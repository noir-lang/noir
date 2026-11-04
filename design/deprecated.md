# `#[deprecated]`

`#[deprecated]` marks an item callers should stop using. The compiler warns at each use, or
reports an error with `#[deprecated(deny)]`. An optional string replaces the default message.

## Trait methods and their impls

A call on a concrete type (`Bar {}.foo()`, `Bar::foo(x)`) resolves to the impl's method, not to
the trait's. Deprecating a trait method covers those calls too, so the lint also reads the
attribute of the trait method that the callee implements. Implementing a deprecated trait method
does not warn: the impl's author may have to provide it for as long as the trait requires it.

Rust behaves the same way for these calls.

## When the impl method is deprecated too

The two attributes are merged, not ranked:

- `deny` on either one makes the call an error.
- The impl method's message is used, falling back to the trait method's.

Merging means a trait author's hard error holds for every impl, including one that adds a plain
`#[deprecated]` of its own. This matters beyond `deny` as written in source, because
`FunctionDefinition::disable` is implemented as `deprecated(deny)` on the disabled function, which
a macro can apply to a trait method (see the `comptime_trait_method_disable` test program).

Rust rejects `#[deprecated]` on the items of a trait impl (`useless_deprecated`), so it has no
such case. Noir accepts it and applies it to calls that resolve to that impl method.

## `#[allow]`

Deprecation is not a lint that `#[allow]` can name (see [lints.md](./lints.md)), so its warnings
can't be silenced at the use site, unlike Rust's `#[allow(deprecated)]`.
