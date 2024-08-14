#[foo]
struct Foo {}

fn main() {
    assert_eq(ONE, 1);
    assert_eq(TWO, 2);
}

comptime fn foo(_: StructDefinition) -> Quoted {
    quote {
        global ONE = 1;
        global TWO = 2;
    }
}
