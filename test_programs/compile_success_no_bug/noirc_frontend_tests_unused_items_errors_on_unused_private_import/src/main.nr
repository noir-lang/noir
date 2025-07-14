
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    use foo::bar;
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    