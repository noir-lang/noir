impl <let N: u32, T> fmtstr<N, T> {
    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents

    #[builtin(fmtstr_quoted)]
    // docs:start:quoted
    fn quoted(self) -> Quoted {}
    // docs:end:quoted
}
