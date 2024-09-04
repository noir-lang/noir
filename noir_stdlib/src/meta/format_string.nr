impl <let N: u32, T> fmtstr<N, T> {
    #[builtin(fmtstr_contents)]
    // docs:start:contents
    fn contents(self) -> Quoted {}
    // docs:end:contents
}
