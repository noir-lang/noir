trait Default {
    fn default() -> Self;
}

struct Foo {
}

// This should fail to compile as `default()` should return `Foo`
impl Default for Foo {
    fn default() -> Field {
        x
    }
}

fn main() {
}
