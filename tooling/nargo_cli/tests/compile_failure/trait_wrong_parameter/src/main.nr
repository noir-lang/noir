trait FromField {
    fn default(x: Field) -> Self;
}

struct Foo {
    bar: Field,
}

impl Default for Foo {
    fn default(x: u32) -> Self {
    }
}

fn main() {
}
