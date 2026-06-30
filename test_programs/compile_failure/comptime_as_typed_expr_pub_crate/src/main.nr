// `pub(crate)` items must not be reachable across crates via
// `FunctionDefinition::as_typed_expr`.
fn main() {
    let value = comptime {
        let bar = quote { foo::bar }.as_module().unwrap();
        let mut found = Option::none();
        for f in bar.functions() {
            if f.name() == quote { crate_only } {
                found = Option::some(f);
            }
        }
        let crate_only_expr = found.unwrap().as_typed_expr();
        quote { $crate_only_expr() }
    };

    assert(value == 42);
}
