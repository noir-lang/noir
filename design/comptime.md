# Ordering

Attributes in comptime code run in order from top to the bottom of a file. Attributes
from sub-modules are run before their parent modules, and attributes from sibling modules
are run in the order their modules are declared in the parent module.

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

The comptime `hash` builtins (`Type::hash`, `Quoted::hash`, etc.) must not depend on the
internals of the Rust standard library: `std`'s `DefaultHasher` uses an algorithm that is
explicitly unspecified and may change between Rust releases, so two builds of the same Noir
compiler version made with different Rust toolchains could otherwise produce different hashes
for identical inputs.

These builtins therefore use `DeterministicHasher` (see `builtin_helpers.rs`), a wrapper over
a SipHash-1-3 keyed with a fixed, publicly specified key. SipHash is chosen over a plain
multiplicative hash (e.g. FNV) so that the keyed mixing structure is preserved and collisions
cannot be forged by trivial arithmetic. The integer-write methods are overridden so `usize`
hashes at a fixed 64-bit width and `u128` uses an explicit little-endian encoding; this keeps
the result identical across targets of differing pointer width, e.g. native 64-bit vs the
wasm32 compiler build.

Hash stability across compiler versions is not guaranteed, but a given version must produce
the same hash in every build environment.
