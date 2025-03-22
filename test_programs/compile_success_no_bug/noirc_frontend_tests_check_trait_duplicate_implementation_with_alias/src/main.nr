
    trait Default {
    }

    struct MyStruct {
    }

    type MyType = MyStruct;

    impl Default for MyStruct {
    }

    impl Default for MyType {
    }

    fn main() {
        let _ = MyStruct {}; // silence MyStruct never constructed warning
    }
    