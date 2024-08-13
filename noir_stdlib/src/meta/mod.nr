use crate::collections::umap::UHashMap;
use crate::hash::BuildHasherDefault;
use crate::hash::poseidon2::Poseidon2Hasher;

mod expr;
mod function_def;
mod module;
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

/// `make_impl` is a helper function to make a simple impl, usually while deriving a trait.
/// This impl has a couple assumptions:
/// 1. The impl only has one function, with the signature `function_signature`
/// 2. The trait itself does not have any generics.
///
/// While these assumptions are met, `make_impl` will create an impl from a StructDefinition,
/// automatically filling in the required generics from the struct, along with the where clause.
/// The function body is created by mapping each field with `for_each_field` and joining the
/// results with `join_fields_with`. The result of this is passed to the `body` function for
/// any final processing - e.g. wrapping each field in a `StructConstructor { .. }` expression.
///
/// See `derive_eq` and `derive_default` for example usage.
pub comptime fn make_trait_impl<Env1, Env2>(
    s: StructDefinition,
    trait_name: Quoted,
    function_signature: Quoted,
    for_each_field: fn[Env1](Quoted) -> Quoted,
    join_fields_with: Quoted,
    body: fn[Env2](Quoted) -> Quoted
) -> Quoted {
    let typ = s.as_type();
    let impl_generics = s.generics().map(|g| quote { $g }).join(quote {,});
    let where_clause = s.generics().map(|name| quote { $name: $trait_name }).join(quote {,});

    // `for_each_field(field1) $join_fields_with for_each_field(field2) $join_fields_with ...`
    let fields = s.fields().map(
        |f: (Quoted, Type)| {
        let name = f.0;
        for_each_field(name)
    }
    );
    let body = body(fields.join(join_fields_with));

    quote {
        impl<$impl_generics> $trait_name for $typ where $where_clause {
            $function_signature {
                $body
            }
        }
    }
}
