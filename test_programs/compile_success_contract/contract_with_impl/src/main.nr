contract Foo {
    pub struct T { x: [Field] }

    impl T {
        fn t(self) {
            let _ = self;
        }
    }
}
