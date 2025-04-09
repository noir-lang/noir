
        trait Foo {
            unconstrained fn identity(self) -> Self {
                self
            }

            unconstrained fn foo(self) -> Field;
        }

        impl Foo for u64 {
            unconstrained fn foo(self) -> Field {
                self as Field
            }
        }

        unconstrained fn main() {
            assert_eq(2.foo(), 2.identity() as Field);
        }
    