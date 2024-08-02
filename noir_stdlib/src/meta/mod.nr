use crate::collections::umap::UHashMap;
use crate::hash::BuildHasherDefault;
use crate::hash::poseidon2::Poseidon2Hasher;

mod struct_def;
mod trait_constraint;
mod trait_def;
mod typ;
mod quoted;

/// Calling unquote as a macro (via `unquote!(arg)`) will unquote
/// its argument. Since this is the effect `!` already does, `unquote`
/// itself does not need to do anything besides return its argument.
pub comptime fn unquote(code: Quoted) -> Quoted {
    code
}

/// Returns the type of any value
#[builtin(type_of)]
pub comptime fn type_of<T>(x: T) -> Type {}

type DeriveFunction = fn(StructDefinition) -> Quoted;

comptime mut global HANDLERS: UHashMap<TraitDefinition, DeriveFunction, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

#[varargs]
pub comptime fn derive(s: StructDefinition, traits: [TraitDefinition]) -> Quoted {
    let mut result = quote {};

    for trait_to_derive in traits {
        let handler = HANDLERS.get(trait_to_derive);
        assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

        let trait_impl = handler.unwrap()(s);
        result = quote { $result $trait_impl };
    }

    result
}

unconstrained pub comptime fn derive_via(t: TraitDefinition, f: DeriveFunction) {
    HANDLERS.insert(t, f);
}
