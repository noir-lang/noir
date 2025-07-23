use super::ctstring::AsCtString;

impl<let N: u32, T> fmtstr<N, T> {
    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    pub comptime fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents

    // docs:start:as_quoted_str
    pub comptime fn as_quoted_str(self) -> Quoted {
    // docs:end:as_quoted_str
        self.as_ctstring().as_quoted_str()
    }
}
