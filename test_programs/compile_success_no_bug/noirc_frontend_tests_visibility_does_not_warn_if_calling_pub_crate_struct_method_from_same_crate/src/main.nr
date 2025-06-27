
    mod moo {
        pub struct Foo {}

        impl Foo {
            pub(crate) fn bar(self) { }
        }
    }

    pub fn method(foo: moo::Foo) {
        foo.bar()
    }

    fn main() {}
    