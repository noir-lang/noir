
    mod foo {
        pub struct Foo {
            foo_inner: Field,
        }
    }

    use foo::Foo;

    #[generate_inner_accessor]
    struct Bar {
        bar_inner: Foo,
    }

    comptime fn generate_inner_accessor(_s: TypeDefinition) -> Quoted {
        quote {
            fn bar_get_foo_inner(x: Bar) -> Field {
                x.bar_inner.foo_inner
            }
        }
    }

    fn main(x: Bar) {
        let _ = bar_get_foo_inner(x);
    }
    