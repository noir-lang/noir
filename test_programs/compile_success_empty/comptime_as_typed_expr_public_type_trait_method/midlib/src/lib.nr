// This macro runs while elaborating the downstream binary, which does NOT depend on
// `leaflib` directly (only transitively, through this crate). `Public` is a public type,
// so resolving its trait method through `as_typed_expr` must be allowed regardless of the
// caller's direct-dependency set.
pub comptime fn generate(_f: FunctionDefinition) -> Quoted {
    let public_type = quote { leaflib::Public }.as_type();
    let constraint = quote { leaflib::Compute }.as_trait_constraint();
    let compute_impl = public_type.get_trait_impl(constraint).unwrap();
    let compute = compute_impl.methods()[0].as_typed_expr();
    quote {
        pub fn call_public_trait_method() -> Field {
            $compute()
        }
    }
}
