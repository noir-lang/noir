# `#[deprecated]`

`#[deprecated]` marks an item callers should stop using. The compiler warns at each use, or
reports an error with `#[deprecated(deny)]`. An optional string replaces the default message.

## Trait methods and their impls

A call on a concrete type (`Bar {}.foo()`, `Bar::foo(x)`) resolves to the impl's method, not to
the trait's. Deprecating a trait method covers those calls too, so the lint also reads the
attribute of the trait method that the callee implements. Implementing a deprecated trait method
does not warn: the impl's author may have to provide it for as long as the trait requires it.

`nargo doc` uses the same lookup as the lint, so an impl method whose calls warn shows as deprecated.

Rust behaves the same way for these calls.

## `#[deprecated]` on a trait impl method

The attribute has no effect on a method in a trait impl and warns that it's useless. It is removed
during def collection, so nothing later (calls, `nargo doc`, `nargo expand`, comptime queries) can
observe it. A crate only sees warnings from its own files, so the warning reaches the author of the
impl, not the crates that depend on it.

A trait impl has to provide every required method for as long as the trait requires it, so one impl
can't meaningfully retire a method on its own, and letting it would let an impl soften a trait
method's `deny` into a warning. Rust denies the attribute there by default, through the `useless_deprecated` lint, which
`#[allow(useless_deprecated)]` can silence. Noir warns, so that code which has the attribute keeps
compiling.

Methods in inherent impls are ordinary functions and can be deprecated as usual.

`FunctionDefinition::disable` is a separate path. It is implemented as `deprecated(deny)` applied
after collection and can target a trait impl method, a trait method or a free function. A call
errors if its callee is disabled, or if the trait method its callee implements is (see the
`comptime_trait_method_disable` and `comptime_trait_impl_method_disable` test programs).

## Deprecated traits

Using a deprecated trait warns when it is named in an `impl` header, a bound, a where clause, a parent
trait, an associated type bound, a trait alias target, `impl Trait` or a type such as
`<Bar as Foo>::Assoc`. Naming it in a `use`, `Foo::C` or `let f = Foo::foo;` doesn't warn yet, and neither
does an operator that dispatches to it. Calling one of its methods
warns too, with the two attributes merged rather than ranked: the method's note is used, falling back
to the trait's, and `deny` on either one makes the call an error, so a method can't soften its
trait's hard error. `<Bar as Foo>::foo(x)` warns once,
on the method. As in Rust:

- Uses inside the trait's own declaration are silent, so that its default methods can call each
  other and its methods can bound on it.
- Uses inside the trait's impls and inside other deprecated items warn.

A written bound can be resolved several times (an impl's where clause is resolved for each method,
and a trait alias resolves its bound twice), so each source location is reported once. The impl generated for a trait alias
doesn't count as a use of the alias, which only warns where it is named.

## `#[allow]`

Deprecation is not a lint that `#[allow]` can name (see [lints.md](./lints.md)), so its warnings
can't be silenced at the use site, unlike Rust's `#[allow(deprecated)]`.
