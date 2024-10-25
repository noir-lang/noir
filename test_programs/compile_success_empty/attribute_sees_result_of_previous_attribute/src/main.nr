#[derive(Eq)]
pub struct Foo {}

#[check_eq_derived_for_foo]
fn main() {}

comptime fn check_eq_derived_for_foo(_f: FunctionDefinition) {
    let foo = quote {Foo}.as_type();
    let eq = quote {Eq}.as_trait_constraint();
    assert(foo.implements(eq));
}
