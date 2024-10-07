#[derive(Eq)]
struct S {}

#[attribute]
fn main() {}

comptime fn attribute(_f: FunctionDefinition) {
    let s = quote { S }.as_type();
    let eq = quote { Eq }.as_trait_constraint();
    assert(s.get_trait_impl(eq).is_some());
}
