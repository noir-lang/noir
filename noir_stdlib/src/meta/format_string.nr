use super::ctstring::AsCtString;

impl<let N: u32, T> fmtstr<N, T> {
    /// Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let x = 3_u32;
    ///     assert_eq(f"{x} + 1".quoted_contents(), quote { 3 + 1 });
    /// }
    /// ```
    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    pub comptime fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents

    /// Returns the format string contents (with the leading and trailing double quotes) as a `Quoted` string literal (not a format string literal).
    ///
    /// Values interpolated into the string are formatted the same way as in `println`, so a `Field`
    /// is written in hexadecimal.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let x = 3_u32;
    ///     let message: str<6> = f"x is {x}".as_quoted_str!();
    ///     assert_eq(message, "x is 3");
    /// }
    /// ```
    // docs:start:as_quoted_str
    pub comptime fn as_quoted_str(self) -> Quoted {
        // docs:end:as_quoted_str
        self.as_ctstring().as_quoted_str()
    }
}
