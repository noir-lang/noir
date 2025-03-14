/// Produces a warning during compilation.
///
/// Note: this function cannot be used at runtime.
#[builtin(warn)]
pub comptime fn warn<T, let N: u32>(message: fmtstr<N, T>) {}
