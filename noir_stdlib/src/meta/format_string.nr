use super::ctstring::AsCtString;

impl<let N: u32, T> fmtstr<N, T> {
    /// Returns the format string contents (that is, without the leading and trailing double quotes) as a `Quoted` value.
    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    pub comptime fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents

    /// Returns the format string contents (with the leading and trailing double quotes) as a `Quoted` string literal (not a format string literal).
    // docs:start:as_quoted_str
    pub comptime fn as_quoted_str(self) -> Quoted {
        // docs:end:as_quoted_str
        self.as_ctstring().as_quoted_str()
    }
}
