# Ordering

Attributes in comptime code run in order from top to the bottom of a file. Attributes
from sub-modules are run before their parent modules, and attributes from sibling modules
are run in the order their modules are declared in the parent module.

This is implemented by sorting collected attributes by `(module topological order, span start)`.
That key can tie for macro-generated items, which can share a single source location. Ties are
broken by collection order, so every container iterated during collection must be ordered (issue #12933).

`comptime` blocks within functions are affected by function's arbitrary elaboration order.
Since functions are lazily elaborated now, this will affect ordering of comptime blocks as well.

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

# Hashing of comptime items

The comptime `hash` builtins (`Type::hash`, `Quoted::hash`, etc.) make no guarantee that an
item hashes to the same value from one compiler version to the next. They do guarantee that a
single compiler version produces the same hash in every build environment: the hash does not
depend on the Rust toolchain the compiler was built with or the target it runs on, so Noir
code compiles identically across platforms.
