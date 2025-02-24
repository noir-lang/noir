/// For any type, return an instance of that type by initializing
/// all of its fields to 0. This is considered to be unsafe since there
/// is no guarantee that all zeroes is a valid bit pattern for every type.
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
#[builtin(checked_transmute)]
pub fn checked_transmute<T, U>(value: T) -> U {}

/// Returns the internal reference count of an array value in unconstrained code.
///
/// Arrays only have reference count in unconstrained code - using this anywhere
/// else will return zero.
#[builtin(array_refcount)]
pub fn array_refcount<T, let N: u32>(array: [T; N]) -> u32 {}

/// Returns the internal reference count of a slice value in unconstrained code.
///
/// Slices only have reference count in unconstrained code - using this anywhere
/// else will return zero.
#[builtin(slice_refcount)]
pub fn slice_refcount<T>(slice: [T]) -> u32 {}
