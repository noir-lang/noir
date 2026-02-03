mod moo {
    pub trait Trait {
        fn trait_method(self);
    }

    impl Trait for Field {
        fn trait_method(self) {}
    }

    pub comptime fn attr(_: FunctionDefinition) -> Quoted {
        let trait_constraint = quote { Trait }.as_trait_constraint();
        let field = quote { Field }.as_type();
        let trait_impl = field.get_trait_impl(trait_constraint).unwrap();
        let method = trait_impl.methods()[0].as_typed_expr();
        let x = 1;
        quote {
            pub fn bar() {
                let _ = $x;
                $method(1);
            }
        }
    }
}

#[moo::attr]
fn main() {
    bar();
}
