# Accepted types restriction

- Oracles should be able to accept & return function types even though they are lowered into
field values during SSA. This is undocumented but shouldn't break anything. `println` currently
would break if it weren't allowed to accept functions.
