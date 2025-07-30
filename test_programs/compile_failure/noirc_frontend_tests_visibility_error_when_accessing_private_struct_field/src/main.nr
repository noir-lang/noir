
    mod moo {
        pub struct Foo {
            x: Field
        }
    }

    fn foo(foo: moo::Foo) -> Field {
        foo.x
    }

    fn main() {}
    