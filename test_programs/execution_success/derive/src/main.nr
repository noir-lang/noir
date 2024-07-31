// Put derive functions in a separate module to show that TraitConstraints
// on an attribute can be resolved in the caller's scope
mod derive {
    use std::collections::umap::UHashMap;
    use std::hash::BuildHasherDefault;
    use std::hash::poseidon2::Poseidon2Hasher;

    type DeriveFunction = fn(StructDefinition) -> Quoted;

    comptime mut global HANDLERS: UHashMap<TraitDefinition, DeriveFunction, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

    #[varargs]
    comptime fn my_derive(s: StructDefinition, traits: [TraitDefinition]) -> Quoted {
        let mut result = quote {};

        for trait_to_derive in traits {
            let handler = HANDLERS.get(trait_to_derive);
            assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

            let trait_impl = handler.unwrap()(s);
            result = quote { $result $trait_impl };
        }

        result
    }

    unconstrained comptime fn my_derive_via(t: TraitDefinition, f: DeriveFunction) {
        HANDLERS.insert(t, f);
    }
}

#[derive::my_derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

#[derive::my_derive(DoNothing)]
struct MyStruct { my_field: u32 }

comptime fn derive_do_nothing(s: StructDefinition) -> Quoted {
    let typ = s.as_type();
    let generics = s.generics().join(quote {,});
    quote {
        impl<$generics> DoNothing for $typ {
            fn do_nothing(_self: Self) {
                // Traits can't tell us what to do
                println("something");
            }
        }
    }
}

fn main() {
    let s = MyStruct { my_field: 1 };
    s.do_nothing();
}
