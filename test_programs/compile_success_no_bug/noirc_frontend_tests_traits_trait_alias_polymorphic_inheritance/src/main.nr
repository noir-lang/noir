
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar<T> {
            fn bar(self) -> T;
        }

        trait Baz<T> = Foo + Bar<T>;

        fn baz<T, U>(x: T) -> U where T: Baz<U> {
            x.foo().bar()
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

        fn main() {
            assert(0.foo().bar() == baz(0));
        }