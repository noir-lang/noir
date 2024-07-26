use std::collections::umap::UHashMap;
use std::hash::BuildHasherDefault;
use std::hash::poseidon2::Poseidon2Hasher;

#[my_derive(DoNothing)]
struct MyStruct { my_field: u32 }

type DeriveFunction = fn(StructDefinition) -> Quoted;

comptime mut global HANDLERS: UHashMap<TraitConstraint, DeriveFunction, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

comptime fn my_derive(s: StructDefinition, traits: [Quoted]) -> Quoted {
    let mut result = quote {};

    for trait_to_derive in traits {
        let handler = HANDLERS.get(trait_to_derive.as_trait_constraint());
        assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

        let trait_impl = handler.unwrap()(s);
        result = quote { $result $trait_impl };
    }

    result
}

unconstrained comptime fn my_derive_via(t: TraitDefinition, f: Quoted) {
    HANDLERS.insert(t.as_trait_constraint(), std::meta::unquote!(f));
}

#[my_derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

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
