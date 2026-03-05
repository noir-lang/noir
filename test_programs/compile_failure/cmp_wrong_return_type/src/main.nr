struct Foo {}

// Intentionally incorrect Ord impl that returns wrong type
impl std::cmp::Ord for Foo {
    fn cmp(self, _other: Self) -> std::cmp::Ordering {
        // Returns unit instead of Ordering
        ()
    }
}

fn main() {
    comptime {
        let a = Foo {};
        let b = Foo {};
        let _ = a < b;
    }
}
