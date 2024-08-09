#[trait_attr]
trait Foo {
    fn foo(self) -> Self;
}

comptime fn trait_attr(_t: TraitDefinition) -> Quoted {
    quote {
        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }
    }
}

fn main() {
    assert_eq(1.foo(), 2);
    assert_eq(10.foo(), 11);
}
