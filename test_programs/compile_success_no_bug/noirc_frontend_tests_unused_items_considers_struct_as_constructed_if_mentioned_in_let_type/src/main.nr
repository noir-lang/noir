
    struct Bar {}

    fn foo(array: [Bar; 1]) {
        let _: Bar = array[0];
    }

    fn main() {
        let _ = foo;
    }
    