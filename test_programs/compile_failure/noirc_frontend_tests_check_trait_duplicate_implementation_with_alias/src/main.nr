
    trait Default2 {
    }

    struct MyStruct {
    }

    type MyType = MyStruct;

    impl Default2 for MyStruct {
    }

    impl Default2 for MyType {
    }

    fn main() {
        let _ = MyStruct {}; // silence MyStruct never constructed warning
    }
    