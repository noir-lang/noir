/// An identity function that *hints* to the compiler to be maximally pessimistic about what `black_box` could do.
///
/// This can be used to block the SSA optimization passes being applied to a value, which should help to prevent
/// test programs from being optimized down to nothing and have them resemble runtime code more closely.
#[builtin(black_box)]
pub fn black_box<T>(value: T) -> T {}
