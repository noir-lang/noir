
        pub trait Foo {
            fn foo(self) -> Self;
        }

        pub trait Bar {
            fn bar(self) -> Self;
        }

        pub trait Baz = Foo + Bar;

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar for Field {
            fn bar(self) -> Self {
                self + 2
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }