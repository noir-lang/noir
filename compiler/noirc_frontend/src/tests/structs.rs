//! Tests for struct definitions and their method implementations.

use crate::{
    elaborator::UnstableFeature,
    tests::{
        assert_no_errors, check_errors, check_errors_using_features, check_monomorphization_error,
        get_program_using_features,
    },
};

#[test]
fn duplicate_struct_field() {
    let src = r#"
    pub struct Foo {
        x: i32,
        ~ First struct field found here
        x: i32,
        ^ Duplicate definitions of struct field with name x found
        ~ Second struct field found here
    }
    "#;
    check_errors(src);
}

#[test]
fn object_type_must_be_known_in_method_call() {
    let src = r#"
    pub fn foo<let N: u32>() -> [Field; N] {
        let array = [];
        let mut bar = array[0];
        let _ = bar.len();
                ^^^ Object type is unknown in method call
                ~~~ Type must be known by this point to know which method to call
        bar
    }
    "#;
    check_errors(src);
}

#[test]
fn incorrect_generic_count_on_struct_impl() {
    let src = r#"
    struct Foo {}
    impl <T> Foo<T> {}
             ^^^ Foo expects 0 generics but 1 was given
    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn uses_self_type_for_struct_function_call() {
    let src = r#"
    struct S { }

    impl S {
        fn one() -> Field {
            1
        }

        fn two() -> Field {
            Self::one() + Self::one()
        }
    }

    fn main() {
        let _ = S {}; // silence S never constructed warning
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_with_better_message_when_trying_to_invoke_struct_field_that_is_a_function() {
    let src = r#"
        pub struct Foo {
            wrapped: fn(Field) -> bool,
        }

        impl Foo {
            fn call(self) -> bool {
                self.wrapped(1)
                ^^^^^^^^^^^^^^^ Cannot invoke function field 'wrapped' on type 'Foo' as a method
                ~~~~~~~~~~~~~~~ to call the function stored in 'wrapped', surround the field access with parentheses: '(', ')'
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn check_impl_duplicate_method_without_self() {
    let src = "
    pub struct Foo {}

    impl Foo {
        fn foo() {}
           ~~~ Previous impl defined here
        fn foo() {}
           ^^^ Impl for type `Foo` overlaps with existing impl
           ~~~ Overlapping impl
    }
    ";
    check_errors(src);
}

#[test]
fn unconstrained_type_parameter_in_impl() {
    let src = r#"
        pub struct Foo<T> {}

        impl<T, U> Foo<T> {}
                ^ The type parameter `U` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `U` type parameter

        fn main() {
            let _ = Foo::<i32> {};
        }
        "#;
    check_errors(src);
}

#[test]
fn unconstrained_numeric_generic_in_impl() {
    let src = r#"
        pub struct Foo {}

        impl<let N: u32> Foo {}
                 ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                 ~ Hint: remove the `N` type parameter

        fn main() {
            let _ = Foo {};
        }
        "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_function_call_for_generic_impl() {
    let src = r#"
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {}
    }

    fn main() {
        Foo::one();
             ^^^ Type annotation needed
             ~~~ Could not determine the type of the generic argument `T` declared on the struct `Foo`
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_type_of_generic_argument_in_struct_constructor() {
    let src = r#"
    struct Foo<T> {}

    fn main()
    {
        let _ = Foo {};
                ^^^ Type annotation needed
                ~~~ Could not determine the type of the generic argument `T` declared on the struct `Foo`
    }

    "#;
    check_errors(src);
}

#[test]
fn resolves_generic_type_argument_via_self() {
    let src = "
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {
            Self::two();
        }

        fn two() {}
    }

    fn main() {
        Foo::<i32>::one();
    }
    ";
    check_monomorphization_error(src);
}

#[test]
fn mutable_self_call() {
    let src = r#"
    fn main() {
        let mut bar = Bar {};
        let _ = bar.bar();
    }

    struct Bar {}

    impl Bar {
        fn bar(&mut self) {
            let _ = self;
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_on_impl_for_type_that_does_not_exist() {
    let src = r#"
    // Try to impl methods on a type that does not exist
    impl Foo {
         ^^^ Could not resolve 'Foo' in path
        fn foo() {}
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_impl_for_primitive_type_outside_stdlib() {
    let src = r#"
    // User code (not stdlib) cannot impl methods on primitive types
    impl Field {
         ^^^^^ Cannot define inherent `impl` for primitive types
         ~~~~~ Primitive types can only have implementation methods defined in the standard library
        fn my_method(self) -> Field {
            self
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn cross_module_impl_resolves_in_impl_module() {
    let src = r#"
    mod types {
        pub struct Point {
            pub x: Field,
            pub y: Field,
        }
    }

    mod methods {
        use crate::types::Point;

        fn helper() -> Field {
            42
        }

        impl Point {
            pub fn distance(self) -> Field {
                // This should resolve helper from the methods module
                helper() + self.x
            }
        }
    }

    use crate::types::Point;

    fn main() {
        let p = Point { x: 1, y: 2 };
        // Method should be accessible via qualified call
        let _ = Point::distance(p);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn constructor_missing_field() {
    let src = r#"
        struct Foo {
            x: Field,
            y: Field,
            z: Field,
        }

        fn main() {
            let _ = Foo { x: 1, y: 2 };
                    ^^^ missing field z in struct Foo
        }
    "#;
    check_errors(src);
}

#[test]
fn constructor_extra_field() {
    let src = r#"
        struct Foo {
            x: Field,
            y: Field,
        }

        fn main() {
            let _ = Foo { x: 1, y: 2, z: 3 };
                                      ^ no such field z defined in struct Foo
        }
    "#;
    check_errors(src);
}

#[test]
fn constructor_private_field() {
    let src = r#"
        mod foo {
            pub struct Foo {
                pub x: Field,
                y: Field,
            }

            pub fn make() -> Foo {
                Foo { x: 1, y: 2 }
            }
        }

        fn main() {
            let f = foo::make();
            let foo::Foo { x: _, y: _ } = f;
                                 ^ y is private and not visible from the current module
                                 ~ y is private
            let foo::Foo { x: _ } = f;
                ^^^^^^^^^^^^^^^^^ missing field y in struct Foo
        }
    "#;
    // NOTE: The second attempt could work with `foo::Foo { x: _, .. }` if Noir supported `..`.
    check_errors(src);
}

#[test]
fn abi_attribute_outside_contract() {
    let src = r#"
        pub contract moo {
            #[abi(hello)]
            pub struct Foo {}
                       ^^^ #[abi(tag)] attributes can only be used in contracts
                       ~~~ misplaced #[abi(tag)] attribute
        }

        pub fn foo(_: moo::Foo) {}
                      ~~~~~~~~ the type is used outside of a contract

        fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn overlapping_inherent_impls() {
    let src = r#"
        struct Foo<T> { _x: T }

        impl<T> Foo<T> {
            fn method(_self: Self) {}
               ^^^^^^ Impl for type `Foo<i32>` overlaps with existing impl
               ~~~~~~ Overlapping impl
        }

        impl Foo<i32> {
            fn method(_self: Self) {}
               ~~~~~~ Previous impl defined here
        }

        fn main() {
            let _ = Foo { _x: 1 };
        }
    "#;
    check_errors(src);
}

#[test]
fn non_overlapping_inherent_impls() {
    // Different concrete types don't overlap
    let src = r#"
        struct Foo<T> { _x: T }

        impl Foo<i32> {
            fn method(_self: Self) {}
        }

        impl Foo<u64> {
            fn method(_self: Self) {}
        }

        fn main() {
            let _ = Foo { _x: 1_i32 };
            let _ = Foo { _x: 1_u64 };
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn deny_cyclic_structs() {
    let src = r#"
    pub struct Foo {
        bar: Bar,
    }

    pub struct Bar {
               ^^^ Dependency cycle found
               ~~~ 'Bar' recursively depends on itself: Bar -> Foo -> Bar
        foo: Foo,
    }

    // Here we check if `Foo` contains references, and this check could
    // cause a stack overflow unless we properly track visited data types.
    pub unconstrained fn foo() -> [Foo; 0] {
        []
    }

    // Here we check the input and output types
    fn main(_: Foo) -> pub [Foo; 0] {
        // Here we also check if the call returns a function, another check
        // that must be done with care.
        // Safety:
        let _ = unsafe { foo() };
        []
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_as_type_non_overlapping() {
    // Test that TraitAsType doesn't cause issues with overlap detection
    // when used in non-overlapping contexts
    let src = r#"
        trait MyTrait<T> {}

        struct MyImpl<T> { _x: T }
        impl<T> MyTrait<T> for MyImpl<T> {}

        struct Foo<T> { _x: T }

        // These impls should not overlap - one uses i32, the other uses u64
        impl Foo<i32> {
            fn method(_self: Self) -> impl MyTrait<i32> {
                // This return type is TraitAsType with NamedGeneric inside
                MyImpl { _x: 0 }
            }
        }

        impl Foo<u64> {
            fn method(_self: Self) -> impl MyTrait<u64> {
                MyImpl { _x: 0 }
            }
        }

        fn main() {
            let _ = Foo { _x: 1_i32 };
            let _ = Foo { _x: 1_u64 };
        }
    "#;
    let features = vec![UnstableFeature::TraitAsType];
    let errors = get_program_using_features(src, &features).2;
    assert!(errors.is_empty(), "Expected no errors but got: {errors:?}");
}

#[test]
fn trait_as_type_overlapping() {
    // Test that overlapping impls are correctly detected even when
    // methods return TraitAsType
    let src = r#"
        trait MyTrait<T> {}

        struct MyImpl<T> { _x: T }
        impl<T> MyTrait<T> for MyImpl<T> {}

        struct Foo<T> { _x: T }

        impl<T> Foo<T> {
            fn method(_self: Self) -> impl MyTrait<T> {
               ^^^^^^ Impl for type `Foo<i32>` overlaps with existing impl
               ~~~~~~ Overlapping impl
                // This return type is TraitAsType with NamedGeneric inside
                MyImpl { _x: _self._x }
            }
        }

        impl Foo<i32> {
            fn method(_self: Self) -> impl MyTrait<i32> {
               ~~~~~~ Previous impl defined here
                MyImpl { _x: _self._x }
            }
        }

        fn main() {
            let _ = Foo { _x: 1_i32 };
        }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn returns_trait_as_type() {
    // Test TraitAsType with more complex generic nesting
    let src = r#"
        trait MyTrait<T, U> {}

        struct MyImpl<T, U> { _x: T, _y: U }
        impl<T, U> MyTrait<T, U> for MyImpl<T, U> {}

        struct Container<T> { _x: T }

        // Non-overlapping impls with TraitAsType in return position
        impl Container<i32> {
            fn get(_self: Self) -> impl MyTrait<i32, Field> {
                MyImpl { _x: _self._x, _y: 0 }
            }
        }

        impl Container<u64> {
            fn get(_self: Self) -> impl MyTrait<u64, bool> {
                MyImpl { _x: _self._x, _y: false }
            }
        }

        fn main() {
            let _ = Container { _x: 1_i32 };
            let _ = Container { _x: 1_u64 };
        }
    "#;
    let features = vec![UnstableFeature::TraitAsType];
    let errors = get_program_using_features(src, &features).2;
    assert!(errors.is_empty(), "Expected no errors but got: {errors:?}");
}

#[test]
fn returns_trait_as_type_overlap() {
    // This test demonstrates that TraitAsType doesn't need special handling in
    // instantiate_named_generics() because:
    // 1. TraitAsType can only appear in method signatures (return/parameter types)
    // 2. Overlap detection only examines the impl target type (e.g., Foo<T> vs Foo<i32>)
    // 3. Method signatures don't affect whether two impls overlap
    //
    // Here, even though the methods have different TraitAsType signatures,
    // the impls still overlap because both could apply to Foo<i32>.
    let src = r#"
        trait Trait1<T> {}
        trait Trait2<T> {}

        struct Impl1<T> { _x: T }
        impl<T> Trait1<T> for Impl1<T> {}

        struct Impl2<T> { _x: T }
        impl<T> Trait2<T> for Impl2<T> {}

        struct Foo<T> { _x: T }

        impl<T> Foo<T> {
            // Returns impl Trait1<T>
            fn method(_self: Self) -> impl Trait1<T> {
               ^^^^^^ Impl for type `Foo<i32>` overlaps with existing impl
               ~~~~~~ Overlapping impl
                Impl1 { _x: _self._x }
            }
        }

        impl Foo<i32> {
            // Returns impl Trait2<i32> - different trait, but still overlaps!
            fn method(_self: Self) -> impl Trait2<i32> {
               ~~~~~~ Previous impl defined here
                Impl2 { _x: _self._x }
            }
        }

        fn main() {
            let _ = Foo { _x: 1_i32 };
        }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}
