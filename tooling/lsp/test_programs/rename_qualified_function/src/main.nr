fn main() -> pub Field {
    foo::bar()
}

mod foo {
    pub fn bar() -> Field {
        1
    }
}
