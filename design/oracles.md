# Accepted types restriction

- Oracles should be able to accept & return function types even though they are lowered into
field values during SSA. This is undocumented but shouldn't break anything. `println` currently
would break if it weren't allowed to accept functions.

# Modifier restrictions

- An `#[oracle(...)]` function must be `unconstrained` (an oracle is resolved by the external
oracle handler at runtime, which only happens in an unconstrained context).
- An `#[oracle(...)]` function cannot be `comptime`. An oracle is resolved at runtime, whereas a
`comptime` function is evaluated at compile time, so the two are fundamentally incompatible.
Without this restriction a `comptime` oracle could still be called at runtime from Brillig (its
empty body bypasses the usual "comptime functions are only callable at compile time" check),
which contradicts the meaning of `comptime`.
