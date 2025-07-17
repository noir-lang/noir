
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    pub(crate) use foo::bar;
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    