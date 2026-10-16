/// True if executed in an unconstrained or comptime context, false otherwise.
///
/// It is resolved at compile time for each context, so a function can use it to pick an
/// implementation: a direct computation in unconstrained code, and a constrained one elsewhere.
///
/// Example:
///
/// ```noir
/// fn context() -> bool {
///     std::runtime::is_unconstrained()
/// }
///
/// unconstrained fn unconstrained_context() -> bool {
///     std::runtime::is_unconstrained()
/// }
///
/// fn main() {
///     assert(!context());
///     // Safety: calling an unconstrained function just to read the result.
///     assert(unsafe { unconstrained_context() });
/// }
/// ```
#[builtin(is_unconstrained)]
pub fn is_unconstrained() -> bool {}
