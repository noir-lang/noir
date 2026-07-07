/// True if executed in an unconstrained or comptime context, false otherwise.
#[builtin(is_unconstrained)]
pub fn is_unconstrained() -> bool {}
