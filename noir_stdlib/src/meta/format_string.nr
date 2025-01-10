impl<let N: u32, T> fmtstr<N, T> {
    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    pub comptime fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents
}
