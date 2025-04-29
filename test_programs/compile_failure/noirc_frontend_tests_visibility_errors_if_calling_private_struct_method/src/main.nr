
    mod moo {
        pub struct Foo {}

        impl Foo {
            fn bar(self) {
                let _ = self;
            }
        }
    }

    pub fn method(foo: moo::Foo) {
        foo.bar()
    }

    fn main() {}
    