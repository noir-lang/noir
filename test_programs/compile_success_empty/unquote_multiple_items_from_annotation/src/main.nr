#[foo]
pub struct Foo {}

fn main() {
    assert_eq(ONE, 1);
    assert_eq(TWO, 2);
}

comptime fn foo(_: StructDefinition) -> Quoted {
    quote {
        global ONE: Field = 1;
        global TWO: u32 = 2;
    }
}
