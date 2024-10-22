// Test that declaring several methods & trait methods with the same name
// does not trigger a duplicate method error
trait ToField {
    fn to_field(self) -> Field;
}
trait ToField2 {
    fn to_field(self) -> Field;
}

pub struct Foo {
    x: Field,
}

impl ToField for Foo {
    fn to_field(self) -> Field {
        self.x
    }
}

impl ToField2 for Foo {
    fn to_field(self) -> Field {
        self.x
    }
}

impl Foo {
    fn to_field(self) -> Field {
        self.x
    }
}

fn main() {}
