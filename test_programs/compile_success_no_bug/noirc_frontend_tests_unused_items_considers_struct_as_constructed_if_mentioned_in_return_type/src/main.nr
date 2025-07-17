
    struct Bar {}

    fn foo(array: [Bar; 1]) -> Bar {
        array[0]
    }

    fn main() {
        let _ = foo;
    }
    