/// Generates an unconstrained version of the given test function.
pub(crate) comptime fn test_unconstrained(f: FunctionDefinition) -> Quoted {
    let name = f.name();
    let unconstrained_name = f"unconstrained_{name}".quoted_contents();
    let body = f.body();
    quote {
        #[test]
        unconstrained fn $unconstrained_name() {
            $body
        }
    }
}
