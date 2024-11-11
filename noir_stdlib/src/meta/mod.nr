pub mod ctstring;
pub mod expr;
pub mod format_string;
pub mod function_def;
pub mod module;
pub mod op;
pub mod struct_def;
pub mod trait_constraint;
pub mod trait_def;
pub mod trait_impl;
pub mod typ;
pub mod typed_expr;
pub mod quoted;
pub mod unresolved_type;

/// Calling unquote as a macro (via `unquote!(arg)`) will unquote
/// its argument. Since this is the effect `!` already does, `unquote`
/// itself does not need to do anything besides return its argument.
// docs:start:unquote
pub comptime fn unquote(code: Quoted) -> Quoted {
    // docs:end:unquote
    code
}

/// Returns the type of any value
#[builtin(type_of)]
// docs:start:type_of
pub comptime fn type_of<T>(x: T) -> Type {}
// docs:end:type_of

// docs:start:derive_example
// These are needed for the unconstrained hashmap we're using to store derive functions
use crate::collections::umap::UHashMap;
use crate::hash::BuildHasherDefault;
use crate::hash::poseidon2::Poseidon2Hasher;

// A derive function is one that given a struct definition can
// create us a quoted trait impl from it.
pub type DeriveFunction = fn(StructDefinition) -> Quoted;

// We'll keep a global HANDLERS map to keep track of the derive handler for each trait
comptime mut global HANDLERS: UHashMap<TraitDefinition, DeriveFunction, BuildHasherDefault<Poseidon2Hasher>> =
    UHashMap::default();

// Given a struct and a slice of traits to derive, create trait impls for each.
// This function is as simple as iterating over the slice, checking if we have a trait
// handler registered for the given trait, calling it, and appending the result.
// docs:start:derive
#[varargs]
pub comptime fn derive(s: StructDefinition, traits: [TraitDefinition]) -> Quoted {
    // docs:end:derive
    let mut result = quote {};

    for trait_to_derive in traits {
        let handler = unsafe { HANDLERS.get(trait_to_derive) };
        assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

        let trait_impl = handler.unwrap()(s);
        result = quote { $result $trait_impl };
    }

    result
}
// docs:end:derive_example

// docs:start:derive_via
// To register a handler for a trait, just add it to our handlers map
// docs:start:derive_via_signature
pub comptime fn derive_via(t: TraitDefinition, f: DeriveFunction) {
    // docs:end:derive_via_signature
    HANDLERS.insert(t, f);
}
// docs:end:derive_via

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
// docs:start:make_trait_impl
pub comptime fn make_trait_impl<Env1, Env2>(
    s: StructDefinition,
    trait_name: Quoted,
    function_signature: Quoted,
    for_each_field: fn[Env1](Quoted) -> Quoted,
    join_fields_with: Quoted,
    body: fn[Env2](Quoted) -> Quoted,
) -> Quoted {
    // docs:end:make_trait_impl
    let typ = s.as_type();
    let impl_generics = s.generics().map(|g| quote { $g }).join(quote {,});
    let where_clause = s.generics().map(|name| quote { $name: $trait_name }).join(quote {,});

    // `for_each_field(field1) $join_fields_with for_each_field(field2) $join_fields_with ...`
    let fields = s.fields().map(|f: (Quoted, Type)| {
        let name = f.0;
        for_each_field(name)
    });
    let body = body(fields.join(join_fields_with));

    quote {
        impl<$impl_generics> $trait_name for $typ where $where_clause {
            $function_signature {
                $body
            }
        }
    }
}

mod tests {
    use crate::meta::derive_via;

    // docs:start:quote-example
    comptime fn quote_one() -> Quoted {
        quote { 1 }
    }

    #[test]
    fn returning_versus_macro_insertion() {
        comptime {
            // let _a: Quoted = quote { 1 };
            let _a: Quoted = quote_one();

            // let _b: Field = 1;
            let _b: Field = quote_one!();

            // Since integers default to fields, if we
            // want a different type we have to explicitly cast
            // let _c: i32 = 1 as i32;
            let _c: i32 = quote_one!() as i32;
        }
    }
    // docs:end:quote-example

    // docs:start:derive-field-count-example
    trait FieldCount {
        fn field_count() -> u32;
    }

    #[derive_field_count]
    struct Bar {
        x: Field,
        y: [Field; 2],
    }

    comptime fn derive_field_count(s: StructDefinition) -> Quoted {
        let typ = s.as_type();
        let field_count = s.fields().len();
        quote {
            impl FieldCount for $typ {
                fn field_count() -> u32 {
                    $field_count
                }
            }
        }
    }
    // docs:end:derive-field-count-example

    // docs:start:annotation-arguments-example
    #[assert_field_is_type(quote { i32 }.as_type())]
    struct MyStruct {
        my_field: i32,
    }

    comptime fn assert_field_is_type(s: StructDefinition, typ: Type) {
        // Assert the first field in `s` has type `typ`
        let fields = s.fields();
        assert_eq(fields[0].1, typ);
    }
    // docs:end:annotation-arguments-example

    // docs:start:annotation-varargs-example
    #[assert_three_args(1, 2, 3)]
    struct MyOtherStruct {
        my_other_field: u32,
    }

    #[varargs]
    comptime fn assert_three_args(_s: StructDefinition, args: [Field]) {
        assert_eq(args.len(), 3);
    }
    // docs:end:annotation-varargs-example

    // docs:start:big-derive-usage-example
    // Finally, to register a handler we call the above function as an annotation
    // with our handler function.
    #[derive_via(derive_do_nothing)]
    trait DoNothing {
        fn do_nothing(self);
    }

    comptime fn derive_do_nothing(s: StructDefinition) -> Quoted {
        // This is simplified since we don't handle generics or where clauses!
        // In a real example we'd likely also need to introduce each of
        // `s.generics()` as well as a trait constraint for each generic
        // to ensure they also implement the trait.
        let typ = s.as_type();
        quote {
            impl DoNothing for $typ {
                fn do_nothing(self) {
                    // Traits can't tell us what to do
                    println("something");
                }
            }
        }
    }

    // Since `DoNothing` is a simple trait which:
    // 1. Only has one method
    // 2. Does not have any generics on the trait itself
    // We can use `std::meta::make_trait_impl` to help us out.
    // This helper function will generate our impl for us along with any
    // necessary where clauses and still provides a flexible interface
    // for us to work on each field on the struct.
    comptime fn derive_do_nothing_alt(s: StructDefinition) -> Quoted {
        let trait_name = quote { DoNothing };
        let method_signature = quote { fn do_nothing(self) };

        // Call `do_nothing` recursively on each field in the struct
        let for_each_field = |field_name| quote { self.$field_name.do_nothing(); };

        // Some traits like Eq want to join each field expression with something like `&`.
        // We don't need that here
        let join_fields_with = quote {};

        // The body function is a spot to insert any extra setup/teardown needed.
        // We'll insert our println here. Since we recur on each field, we should see
        // one println for the struct itself, followed by a println for every field (recursively).
        let body = |body| quote {
            println("something");
            $body
        };
        crate::meta::make_trait_impl(
            s,
            trait_name,
            method_signature,
            for_each_field,
            join_fields_with,
            body,
        )
    }
    // docs:end:big-derive-usage-example

    impl DoNothing for Bar {
        fn do_nothing(_: Self) {}
    }

    // This function is just to remove unused warnings
    fn remove_unused_warnings() {
        let _: Bar = Bar { x: 1, y: [2, 3] };
        let _: MyStruct = MyStruct { my_field: 1 };
        let _: MyOtherStruct = MyOtherStruct { my_other_field: 2 };
        let _ = derive_do_nothing(crate::panic::panic(f""));
        let _ = derive_do_nothing_alt(crate::panic::panic(f""));
        if false {
            remove_unused_warnings();
        }
    }
}
