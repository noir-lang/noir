trait Foo {
    fn to_be_implemented(self) -> Field;

    fn method_that_already_exists(self, other: Self) -> Field
    where
        Self: Eq,
    {
        if self.eq(other) {
            self.to_be_implemented()
        } else {
            0
        }
    }
}

#[derive(Eq)]
pub struct Bar {
    a: Field,
}

impl Foo for Bar {
    fn to_be_implemented(self) -> Field {
        self.a
    }
}

fn main() {}
