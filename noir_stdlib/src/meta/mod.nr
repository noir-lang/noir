mod trait_constraint;
mod trait_def;
mod type_def;
mod quoted;

use crate::collections::umap::UHashMap;
use crate::hash::BuildHasherDefault;
use crate::hash::poseidon2::Poseidon2Hasher;

// `derive` uses `as_trait_constraint` internally which currently resolves
// a trait in the scope of `derive` rather than the caller of `derive`. So to
// allow users to derive traits we have to import them here.
use crate::cmp::{Eq, Ord};
use crate::default::Default;

/// Calling unquote as a macro (via `unquote!(arg)`) will unquote
/// its argument. Since this is the effect `!` already does, `unquote`
/// itself does not need to do anything besides return its argument.
pub comptime fn unquote(code: Quoted) -> Quoted {
    code
}

comptime mut global HANDLERS: UHashMap<TraitConstraint, fn(StructDefinition) -> Quoted, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

pub comptime fn derive(s: StructDefinition, traits: [Quoted]) -> Quoted {
    let mut result = quote {};

    for trait_to_derive in traits {
        let handler = HANDLERS.get(trait_to_derive.as_trait_constraint());
        assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

        let trait_impl = handler.unwrap()(s);
        result = quote { $result $trait_impl };
    }

    result
}

unconstrained pub comptime fn derive_via(t: TraitDefinition, f: Quoted) {
    HANDLERS.insert(t.as_trait_constraint(), unquote!(f));
}
