contract Foo {
    pub struct T {
        x: [Field],
    }

    impl T {
        pub fn t(self) {
            let _ = self;
        }
    }
}
