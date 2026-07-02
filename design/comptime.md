# Ordering

Attributes in comptime code run in order from top to the bottom of a file. Attributes
from sub-modules are run before their parent modules, and attributes from sibling modules
are run in the order their modules are declared in the parent module.

This is implemented by sorting collected attributes by `(module topological order, span start)`.
That key can tie for macro-generated items, which can share a single source location. Ties are
broken by collection order, so every container iterated during collection must be ordered (issue #12933).

`comptime` blocks within functions are affected by function's arbitrary elaboration order.
Since functions are lazily elaborated now, this will affect ordering of comptime blocks as well.

# Attributes on trait methods

Comptime attributes on a trait's default methods run just like attributes on
top-level or impl methods. A comptime attribute on a bodyless trait method declaration has no body
to run against, so it is rejected with an error rather than silently ignored or run.

Attributes on a trait `impl`'s methods run as well, matching inherent `impl` methods, so a bodyless
trait method declaration can instead be given the attribute on each impl. A method inherited from a
trait's default implementation reuses the trait's own `FuncId`, so its attribute is run once at the
trait definition rather than again per inheriting impl.

# Repeat diagnostics

The comptime interpreter can be run on functions which created errors during elaboration.
When this happens, it will execute the function until the error, at which point a repeat
error may be issued. These should be minimized when possible but this needs to be handled
manually in the interpreter by expecting a repeat error variant.

# Mutations and errors

When the comptime interpreter hits an error, it will not revert any mutations it has made
to mutable globals. This can leave these globals in an inconsistent state, potentially leading
to further comptime errors elsewhere.

# Mutating existing items

Comptime functions mutating existing items in the source code should generally be avoided
due to security concerns. These can make auditing code more difficult in particular.

# Splicing resolved types into a quote

A `Type` interpolated into a `quote { .. }` via `$typ` (for example one obtained from
`TypeDefinition::fields_as_written`) is already resolved: its named generics carry the type
variables of the definition the type came from. When such a type is spliced into a new generic
scope — e.g. a generated `impl<Context> Foo<Context> { .. }` — its free (unbound) named generics
are rebound *by name* to the same-named generics in scope at the splice site. This makes a spliced
`$typ` behave like a textually-written type in the quote, whose identifiers resolve by name in the
splice scope; without it, two identically-named generics with different type variables fail to
unify (issue #10747). Rebinding is done in `Elaborator::rebind_resolved_type_generics` when the
elaborator resolves an `UnresolvedTypeData::Resolved`.

Consequences of this rule (macros are name-based, not hygienic, so it matches how a textual
identifier in a quote already resolves):

- Rebinding is **per generic**. A spliced `Pair<X, Y>` in an `impl<X> ..` binds `X` to the impl's
  `X` but leaves `Y` as the origin's generic, so one spliced type can mix captured and origin
  generics. This only surfaces in programs that were already ill-formed (a `Y` that is not in scope
  is a dangling generic either way); it never changes the meaning of a program that compiled before.
- An origin generic whose name is **not** in scope stays bound to its origin (it is not turned into
  a "could not resolve" error the way a textually-written unknown generic name is). This asymmetry
  is pre-existing: spliced types were always frozen before this change; the change only *adds*
  name-capture for the matching case.
- Capture is by name only and `Type`'s `Display` shows only names, so which generic a spliced
  `Context` resolves to is not visible in printed types.
- Only ordinary named generics are rebound. Associated-type and associated-constant projections
  are also modeled as `NamedGeneric`s, but their name *is* the projection (e.g.
  `<T as Deserialize>::N`), so they are excluded (`NamedGeneric::is_associated`). Rebinding one
  would capture it onto a same-named projection at the splice site — e.g. a `#[derive]`-generated
  `let N: u32 = <$field as Deserialize>::N` would tie the field's `<[T; N] as Deserialize>::N` to
  the impl's own `Self::N`, yielding the cyclic associated constant `Self::N = N * Self::N`.

# Hashing of comptime items

The comptime `hash` builtins (`Type::hash`, `Quoted::hash`, etc.) make no guarantee that an
item hashes to the same value from one compiler version to the next. They do guarantee that a
single compiler version produces the same hash in every build environment: the hash does not
depend on the Rust toolchain the compiler was built with or the target it runs on, so Noir
code compiles identically across platforms.
