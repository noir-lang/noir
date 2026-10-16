/// For any type, return an instance of that type by initializing
/// all of its fields to 0. This is considered to be unsafe since there
/// is no guarantee that all zeroes is a valid bit pattern for every type.
///
/// It is useful as a placeholder for values that are overwritten before they are read, for example
/// to fill an array before setting its elements.
///
/// Example:
///
/// ```noir
/// let x: u32 = std::mem::zeroed();
/// assert_eq(x, 0);
///
/// let pair: (bool, [u8; 2]) = std::mem::zeroed();
/// assert_eq(pair, (false, [0, 0]));
/// ```
#[builtin(zeroed)]
pub fn zeroed<T>() -> T {}

/// Transmutes a value of type T to a value of type U.
///
/// Both types are asserted to be equal during compilation but after type checking.
/// If not, a compilation error is issued.
///
/// This function is useful for types using arithmetic generics in cases
/// which the compiler otherwise cannot prove equal during type checking.
/// You can use this to obtain a value of the correct type while still asserting
/// that it is equal to the previous.
///
/// Example:
///
/// ```noir
/// // `concat` returns `[u8; N + N]`, which the type checker doesn't treat as `[u8; 2 * N]`.
/// fn twice<let N: u32>(a: [u8; N]) -> [u8; 2 * N] {
///     std::mem::checked_transmute(a.concat(a))
/// }
///
/// fn main() {
///     assert_eq(twice([1, 2]), [1, 2, 1, 2]);
/// }
/// ```
#[builtin(checked_transmute)]
pub fn checked_transmute<T, U>(value: T) -> U {}

/// Returns the internal reference count of an array value in unconstrained code.
///
/// Arrays only have reference count in unconstrained code - using this anywhere
/// else will return zero.
///
/// It is meant for debugging how often arrays are copied in unconstrained code.
///
/// Example:
///
/// ```noir
/// unconstrained fn main() {
///     let array = [1, 2, 3];
///     assert(std::mem::array_refcount(array) > 0);
/// }
/// ```
#[builtin(array_refcount)]
pub fn array_refcount<T, let N: u32>(array: [T; N]) -> u32 {}

/// Returns the internal reference count of a vector value in unconstrained code.
///
/// Vectors only have reference count in unconstrained code - using this anywhere
/// else will return zero.
///
/// It is meant for debugging how often vectors are copied in unconstrained code.
///
/// Example:
///
/// ```noir
/// unconstrained fn main() {
///     let vector = @[1, 2, 3];
///     assert(std::mem::vector_refcount(vector) > 0);
/// }
/// ```
#[builtin(vector_refcount)]
pub fn vector_refcount<T>(vector: [T]) -> u32 {}
