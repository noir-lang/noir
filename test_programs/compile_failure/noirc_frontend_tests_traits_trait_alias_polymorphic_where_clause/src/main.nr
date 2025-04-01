
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar<T> {
            fn bar(self) -> T;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T> = Foo + Bar<T> where T: Baz;

        fn qux<T, U>(x: T) -> bool where T: Qux<U> {
            x.foo().bar().baz()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar<bool> for Field {
            fn bar(self) -> bool {
                true
            }
        }

        impl Baz for bool {
            fn baz(self) -> bool {
                self
            }
        }

        fn main() {
            assert(0.foo().bar().baz() == qux(0));
        }
    