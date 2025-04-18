
    struct Foo {
        x: Field
    }

    mod nested {
        fn foo(foo: super::Foo) -> Field {
            foo.x
        }
    }

    fn main() {
        let _ = Foo { x: 1 };
    }
    