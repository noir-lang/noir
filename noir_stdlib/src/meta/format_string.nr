impl <let N: u32, T> fmtstr<N, T> {
    // docs:start:quoted
    comptime fn quoted(self) -> Quoted {
        crate::meta::unquote!(quote { $self })
    }
    // docs:end:quoted

    #[builtin(fmtstr_quoted_contents)]
    // docs:start:quoted_contents
    comptime fn quoted_contents(self) -> Quoted {}
    // docs:end:quoted_contents
}
