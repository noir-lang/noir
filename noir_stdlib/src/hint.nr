/// An identity function that *hints* to the compiler to be maximally pessimistic about what `black_box` could do.
///
/// This can be used to block the SSA optimization passes being applied to a value, which should help to prevent
/// test programs from being optimized down to nothing and have them resemble runtime code more closely.
///
/// It returns `value` unchanged and adds no constraints of its own.
///
/// Example:
///
/// ```noir
/// // Without `black_box`, the compiler could compute `x * 2` at compile time.
/// let x = std::hint::black_box(21);
/// assert_eq(x * 2, 42);
/// ```
#[builtin(black_box)]
pub fn black_box<T>(value: T) -> T {}
