
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Baz = Foo;

        impl Foo for Field {
            fn foo(self) -> Self { self }
        }

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo()
        }

        fn main() {
            let x: Field = 0;
            let _ = baz(x);
        }
    