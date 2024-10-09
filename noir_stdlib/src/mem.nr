/// For any type, return an instance of that type by initializing
/// all of its fields to 0. This is considered to be unsafe since there
/// is no guarantee that all zeroes is a valid bit pattern for every type.
#[builtin(zeroed)]
pub fn zeroed<T>() -> T {}

/// Transmutes a value of type T to a value of type U.
///
/// This operation is checked in that in a later stage of compilation
/// both types are asserted to be equal. If not, a compile error is issued.
///
/// This function is useful for types using arithmetic generics for cases
/// which the compiler otherwise cannot prove as equal during type checking.
/// You can use this to obtain a value of the correct type while still asserting
/// that it is equal to the previous.
#[builtin(checked_transmute)]
pub fn checked_transmute<T, U>(value: T) -> U {}
