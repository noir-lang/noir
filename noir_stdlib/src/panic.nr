/// Halt the program at runtime with the given error message.
///
/// The provided error message must be either a `str` or a `fmtstr`.
///
/// It returns any type, so it can stand in for a value in a branch that should never run. It is
/// `assert(false, message)`: in constrained code, the branch that calls it must be unreachable for
/// the program to prove.
///
/// Example:
///
/// ```noir
/// fn digit_value(c: u8) -> u8 {
///     if (c >= 48) & (c <= 57) {
///         c - 48
///     } else {
///         panic(f"not a digit: {c}")
///     }
/// }
///
/// fn main() {
///     assert_eq(digit_value(55), 7);
/// }
/// ```
///
/// Calling it fails the program with `message`:
///
/// ```noir should_fail
/// // Fails with the message "something went wrong".
/// let _: u8 = panic("something went wrong");
/// ```
pub fn panic<T, U>(message: T) -> U
where
    T: StringLike,
{
    assert(false, message);
    crate::mem::zeroed()
}

trait StringLike {}

impl<let N: u32> StringLike for str<N> {}
impl<let N: u32, T> StringLike for fmtstr<N, T> {}

mod tests {
    use crate::prelude::panic;

    #[test(should_fail_with = "OH NO")]
    fn panics() {
        panic("OH NO");
    }
}
