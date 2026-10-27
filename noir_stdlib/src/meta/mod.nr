pub mod ctstring;
pub mod expr;
pub mod format_string;
pub mod function_def;
pub mod item_visibility;
pub mod location;
pub mod module;
pub mod op;
pub mod trait_constraint;
pub mod trait_def;
pub mod trait_impl;
pub mod typ;
pub mod type_def;
pub mod typed_expr;
pub mod quoted;
pub mod unresolved_type;

use crate::default::Default;
use crate::option::Option;

/// Inserts quoted code at the call site when called as `unquote!(code)`.
///
/// Example:
///
/// ```noir
/// fn main() {
///     let sum: u32 = std::meta::unquote!(quote { 1 + 2 });
///     assert_eq(sum, 3);
/// }
/// ```
// docs:start:unquote
pub comptime fn unquote(code: Quoted) -> Quoted {
    // docs:end:unquote
    code
}

/// Returns the type of any value.
///
/// Example:
///
/// ```noir
/// comptime {
///     assert_eq(std::meta::type_of(1_u8), quote { u8 }.as_type());
///     assert_eq(std::meta::type_of([true, false]), quote { [bool; 2] }.as_type());
/// }
/// ```
#[builtin(type_of)]
// docs:start:type_of
pub comptime fn type_of<T>(x: T) -> Type {}
// docs:end:type_of

/// Issues an error diagnostic at `location`, with an optional secondary message.
///
/// Unlike a comptime panic, current execution will continue, allowing multiple errors to be
/// issued.
///
/// Example:
///
/// ```noir
/// #[requires_parameters]
/// pub fn add(x: u32, y: u32) -> u32 {
///     x + y
/// }
///
/// comptime fn requires_parameters(f: FunctionDefinition) {
///     if f.parameters().len() == 0 {
///         let name = f.name();
///         std::meta::error(
///             f"`{name}` must take parameters",
///             Option::some(f"add at least one parameter"),
///             f.location(),
///         );
///     }
/// }
/// ```
#[builtin(issue_error)]
// docs:start:error
pub comptime fn error<let N: u32, T, let N2: u32, T2>(
    _msg: fmtstr<N, T>,
    _secondary: Option<fmtstr<N2, T2>>,
    _location: Location,
) {}
// docs:end:error

/// Issues a warning diagnostic at `location`, with an optional secondary message.
///
/// Execution continues after this function is called.
///
/// Example:
///
/// ```noir
/// #[check]
/// pub fn add(x: u32, y: u32) -> u32 {
///     x + y
/// }
///
/// comptime fn check(f: FunctionDefinition) {
///     // Warns "`add` has no documentation", pointing at `add`.
///     let name = f.name();
///     std::meta::warn(
///         f"`{name}` has no documentation",
///         Option::<fmtstr<0, ()>>::none(),
///         f.location(),
///     );
/// }
/// ```
#[builtin(issue_warning)]
// docs:start:warn
pub comptime fn warn<let N: u32, T, let N2: u32, T2>(
    _msg: fmtstr<N, T>,
    _secondary: Option<fmtstr<N2, T2>>,
    _location: Location,
) {}
// docs:end:warn

/// A derive function: given a type definition, it returns a quoted trait impl for that type.
///
/// Register one for a trait with [`derive_via`], then use the trait in [`derive`].
///
/// Example:
///
/// ```noir
/// use std::meta::DeriveFunction;
///
/// comptime fn derive_nothing(_s: TypeDefinition) -> Quoted {
///     quote {}
/// }
///
/// fn main() {
///     comptime {
///         let _handler: DeriveFunction = derive_nothing;
///     }
/// }
/// ```
// docs:start:derive_example
// A derive function is one that given a type definition can
// create us a quoted trait impl from it.
pub comptime type DeriveFunction = fn(TypeDefinition) -> Quoted;

// These are needed for the unconstrained hashmap we're using to store derive functions
use crate::collections::umap::UHashMap;
use crate::hash::BuildHasherDefault;
use crate::hash::poseidon2::Poseidon2Hasher;

// We'll keep a global HANDLERS map to keep track of the derive handler for each trait
comptime mut global HANDLERS: UHashMap<TraitDefinition, DeriveFunction, BuildHasherDefault<Poseidon2Hasher>> =
    UHashMap::default();
// docs:end:derive_example

/// Implements each trait in `traits` for the type, using the derive functions registered for them.
///
/// Register derive functions with [`derive_via`]. Use it as an attribute: `#[derive(Eq, Default)]`.
///
/// Compilation fails if no derive function is registered for one of the traits.
///
/// Example:
///
/// ```noir
/// #[derive(Eq, Default)]
/// pub struct Point {
///     x: u32,
///     y: u32,
/// }
///
/// fn main() {
///     assert_eq(Point::default(), Point { x: 0, y: 0 });
/// }
/// ```
// docs:start:derive_example_body
// Given a type definition and a vector of traits to derive, create trait impls for each.
// This function is as simple as iterating over the vector, checking if we have a trait
// handler registered for the given trait, calling it, and appending the result.
// docs:start:derive
#[varargs]
pub comptime fn derive(s: TypeDefinition, traits: [TraitDefinition]) -> Quoted {
    // docs:end:derive
    let mut result = quote {};

    for trait_to_derive in traits {
        let handler = HANDLERS.get(trait_to_derive);
        assert(handler.is_some(), f"No derive function registered for `{trait_to_derive}`");

        let trait_impl = handler.unwrap()(s);
        result = quote { $result $trait_impl };
    }

    result
}
// docs:end:derive_example_body

/// Registers `f` as the derive function for the trait `t`, so that `#[derive(T)]` calls it.
///
/// Use it as an attribute on the trait: `#[derive_via(f)]`.
///
/// Compilation fails if a derive function is already registered for the trait.
///
/// Example:
///
/// ```noir
/// #[derive_via(derive_greet)]
/// pub trait Greet {
///     fn greet() -> str<5>;
/// }
///
/// comptime fn derive_greet(s: TypeDefinition) -> Quoted {
///     let typ = s.as_type();
///     quote {
///         impl Greet for $typ {
///             fn greet() -> str<5> {
///                 "hello"
///             }
///         }
///     }
/// }
///
/// #[derive(Greet)]
/// pub struct Person {}
///
/// fn main() {
///     assert_eq(Person::greet(), "hello");
/// }
/// ```
// docs:start:derive_via
// To register a handler for a trait, just add it to our handlers map
// docs:start:derive_via_signature
pub comptime fn derive_via(t: TraitDefinition, f: DeriveFunction) {
    // docs:end:derive_via_signature
    assert(HANDLERS.insert(t, f), f"A derive_via function has already been registered for {t}");
}
// docs:end:derive_via

/// A helper to make a simple trait impl for a type, usually in a derive function.
///
/// It assumes the trait has one method, with the signature `function_signature`, and no generics.
///
/// Under these assumptions, `make_trait_impl` will create an impl from a [`TypeDefinition`],
/// automatically filling in the required generics from the type, along with the where clause.
/// The function body is created by mapping each field with `for_each_field` and joining the
/// results with `join_fields_with`. The result of this is passed to the `body` function for
/// any final processing - e.g. wrapping each field in a `StructConstructor { .. }` expression.
///
/// Example:
///
/// ```noir
/// pub trait IsZero {
///     fn is_zero(self) -> bool;
/// }
///
/// // An attribute that returns quoted code adds that code to the program.
/// #[derive_is_zero]
/// pub struct Point {
///     x: u32,
///     y: u32,
/// }
///
/// comptime fn derive_is_zero(s: TypeDefinition) -> Quoted {
///     // `fn is_zero(self) -> bool { (self.x == 0) & (self.y == 0) }`
///     std::meta::make_trait_impl(
///         s,
///         quote { IsZero },
///         quote { fn is_zero(self) -> bool },
///         |name| quote { (self.$name == 0) },
///         quote { & },
///         |body| body,
///     )
/// }
///
/// fn main() {
///     assert(Point { x: 0, y: 0 }.is_zero());
///     assert(!Point { x: 1, y: 0 }.is_zero());
/// }
/// ```
// docs:start:make_trait_impl
pub comptime fn make_trait_impl<Env1, Env2>(
    s: TypeDefinition,
    trait_name: Quoted,
    function_signature: Quoted,
    for_each_field: fn[Env1](Quoted) -> Quoted,
    join_fields_with: Quoted,
    body: fn[Env2](Quoted) -> Quoted,
) -> Quoted {
    // docs:end:make_trait_impl
    let typ = s.as_type();

    let mut impl_generics = [].as_vector();
    let mut where_clause = [].as_vector();
    for g in s.generics() {
        let typ = g.typ();
        impl_generics = impl_generics.push_back(quote { $typ });
        if g.numeric_type().is_none() {
            where_clause = where_clause.push_back(quote { $typ: $trait_name });
        }
    }

    let impl_generics = impl_generics.join(quote {, });
    let where_clause = where_clause.join(quote {, });

    // `for_each_field(field1) $join_fields_with for_each_field(field2) $join_fields_with ...`
    let fields = s.fields_as_written().map(|field| for_each_field(field.name()));
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
    use crate::meta::ctstring::AsCtString;
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

    comptime fn derive_field_count(s: TypeDefinition) -> Quoted {
        let typ = s.as_type();
        let field_count = s.fields_as_written().len();
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

    comptime fn assert_field_is_type(s: TypeDefinition, typ: Type) {
        // Assert the first field in `s` has type `typ`
        let fields = s.fields([]);
        assert_eq(fields[0].typ(), typ);
    }
    // docs:end:annotation-arguments-example

    // docs:start:annotation-varargs-example
    #[assert_three_args(1, 2, 3)]
    struct MyOtherStruct {
        my_other_field: u32,
    }

    #[varargs]
    comptime fn assert_three_args(_s: TypeDefinition, args: [Field]) {
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

    comptime fn derive_do_nothing(s: TypeDefinition) -> Quoted {
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
    comptime fn derive_do_nothing_alt(s: TypeDefinition) -> Quoted {
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

    // docs:start:concatenate-example
    comptime fn concatenate(q1: Quoted, q2: Quoted) -> Quoted {
        assert(q1.tokens().len() <= 1);
        assert(q2.tokens().len() <= 1);

        f"{q1}{q2}".quoted_contents()
    }

    // The CtString type is also useful for a compile-time string of unbounded size
    // so that you can append to it in a loop.
    comptime fn double_spaced(q: Quoted) -> CtString {
        let mut result = "".as_ctstring();

        for token in q.tokens() {
            if result != "".as_ctstring() {
                result = result.append_str("  ");
            }
            result = result.append_fmtstr(f"{token}");
        }

        result
    }

    #[test]
    fn concatenate_test() {
        comptime {
            let result = concatenate(quote { foo }, quote { bar });
            assert_eq(result, quote { foobar });

            let result = double_spaced(quote {foo bar 3}).as_quoted_str!();
            assert_eq(result, "foo  bar  3");
        }
    }
    // docs:end:concatenate-example

    // This function is just to remove unused warnings
    fn remove_unused_warnings() {
        let _: Bar = Bar { x: 1, y: [2, 3] };
        let _: MyStruct = MyStruct { my_field: 1 };
        let _: MyOtherStruct = MyOtherStruct { my_other_field: 2 };
        let _ = derive_do_nothing(crate::panic::panic(""));
        let _ = derive_do_nothing_alt(crate::panic::panic(""));
        if false {
            remove_unused_warnings();
        }
    }
}
