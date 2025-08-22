
    struct Bar {}

    struct Generic<T> {}

    fn main() {
        let _ = Generic::<Bar> {};
    }
    