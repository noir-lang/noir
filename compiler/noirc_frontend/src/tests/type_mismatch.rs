use crate::test_utils::stdlib_src;
use crate::tests::{check_errors, check_errors_with_stdlib, check_monomorphization_error};

#[test]
fn cmp_wrong_return_type() {
    let src = r#"
    struct Foo {}

    impl crate::cmp::Ord for Foo {
        fn cmp(self, _other: Self) -> crate::cmp::Ordering {
                                      ^^^^^^^^^^^^^^^^^^^^ expected type Ordering, found type ()
                                      ~~~~~~~~~~~~~~~~~~~~ expected Ordering because of return type
            ()
            ~~ () returned here
        }
    }

    fn main() {
        comptime {
            let a = Foo {};
            let b = Foo {};
            let _ = a < b;
                    ^^^^^ Expected `Ordering` but a value of type `()` was given
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ, stdlib_src::ORD]);
}

#[test]
fn option_expect_bad_input() {
    let option_src = "
        pub struct Option<T> {
            _is_some: bool,
            _value: T,
        }

        impl<T> Option<T> {
            pub fn some(_value: T) -> Self {
                Option { _is_some: true, _value }
            }

            pub fn is_some(self) -> bool {
                self._is_some
            }

            pub fn expect<let N: u32, MessageTypes>(self, message: fmtstr<N, MessageTypes>) -> T {
                assert(self.is_some(), message);
                self._value
            }
        }
    ";
    let src = r#"
    fn main() {
        let inner_value = 3;
        let some = Option::some(inner_value);

        assert(some.expect("Should have the value {inner_value}") == 3);
               ^^^^^^^^^^^ Type annotation needed
               ~~~~~~~~~~~ Could not determine the value of the generic argument `N` declared on the function `expect`
               ~~~~~~~~~~~ Could not determine the type of the generic argument `MessageTypes` declared on the function `expect`
                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type fmtstr<_, _>, found type str<35>
    }
    "#;
    check_errors_with_stdlib(src, [option_src]);
}

#[test]
fn associated_constant_bound_with_different_types() {
    let src = r#"
    trait TraitWithAssociatedConstant {
        let N: u32;

        fn foo(_: Self) -> bool {
            true
        }

        fn make_array(self) -> [Field; Self::N];
    }

    struct Foo {}

    impl TraitWithAssociatedConstant for Foo {
        let N: u32 = 5;

        fn make_array(self) -> [Field; Self::N] {
            crate::zeroed()
        }
    }

    struct Wrapper<T> {
        inner: T,
    }

    impl<U> crate::Eq for Wrapper<U>
    where
        U: TraitWithAssociatedConstant,
    {
        fn eq(self, _other: Self) -> bool {
            let _array1: [Field; 5] = self.inner.make_array();
                                      ^^^^^^^^^^^^^^^^^^^^^^^ Expected type [Field; 5], found type [Field; <U as TraitWithAssociatedConstant>::N]
            let _array2: [Field; 6] = self.inner.make_array();
                                      ^^^^^^^^^^^^^^^^^^^^^^^ Expected type [Field; 6], found type [Field; <U as TraitWithAssociatedConstant>::N]
            self.inner.foo()
        }
    }

    fn main() {
        let wrapper = Wrapper { inner: Foo {} };
        assert_eq(wrapper, wrapper);
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ, stdlib_src::ZEROED]);
}

#[test]
fn bool_math() {
    let src = r#"
    fn main() {
        let _ = true + true;
                ^^^^^^^^^^^ Cannot add a `bool` to a `bool
        let _ = true - true;
                ^^^^^^^^^^^ Cannot subtract a `bool` from a `bool
        let _ = true * true;
                ^^^^^^^^^^^ Cannot multiply a `bool` by a `bool
        let _ = true / true;
                ^^^^^^^^^^^ Cannot divide a `bool` by a `bool`
        let _ = true % true;
                ^^^^^^^^^^^ Cannot calculate the remainder of a `bool` divided by a `bool`
        let _ = true >> true;
                ^^^^^^^^^^^^ No implementation for `bool >> bool`
        let _ = true << true;
                ^^^^^^^^^^^^ No implementation for `bool << bool`
    }
    "#;
    check_errors(src);
}

#[test]
fn field_modulo() {
    let src = r#"
    fn main(x: Field) -> pub Field {
        x % 2
        ^^^^^ Cannot do modulo on Fields, try casting to an integer first
    }
    "#;
    check_errors(src);
}

#[test]
fn unary_not_on_field_type_variable() {
    let src = r#"
    fn main() {
        let num: Field = 0;
        assert_eq(!0, num);
                  ^^^^^^^ Types in a binary operation should match, but found Field and u32
    }
    "#;
    check_errors(src);
}

#[test]
fn integer_too_large() {
    let src = r#"
    fn main(x: Field) {
        let too_large: Field = 233149999999999999999999999999999999999999999999999999999999923314999999999999999999999999999999999999999999999999999999999923314999999999999999999999999999999999999999999999999999999999;
                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Integer literal is too large
                               ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ value exceeds limit of 21888242871839275222246405745257275088548364400416034343698204186575808495616
        assert(x == too_large);
    }
    "#;
    check_errors(src);
}

#[test]
fn for_loop_range_type_mismatch() {
    let src = r#"
    fn main() {
        comptime {
            for _ in 1_i8..2_u8 {}
                           ^^^^ Expected type i8, found type u8
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn generics_on_integer_type() {
    let src = r#"
    fn main() {
        let _: i32<bool> = 1;
               ^^^ i32 expects 0 generics but 1 was given
        let _ = i32::<bool>::default();
                             ^^^^^^^ Could not resolve 'default' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn mutability_regression_2911() {
    let src = r#"
    fn main() {
        let vector: &mut [Field] = &mut [];
                                   ^^^^^^^ Expected type &mut [Field], found type &mut [_; 0]
                                        ^^ Type annotation needed
                                        ~~ Could not determine the type of the array
        vector = vector;
        ^^^^^^ Variable `vector` must be mutable to be assigned to
    }
    "#;
    check_errors(src);
}

#[test]
fn negate_unsigned() {
    let src = r#"
    fn main() {
        let _var = -(1 as u8);
                   ^^^^^^^^^^ Cannot apply unary operator `-` to type `u8`
    }
    "#;
    check_errors(src);
}

#[test]
fn tuple_mismatch() {
    let src = r#"
    fn main() {
        let (_x, _y) = (1, 2, 3);
            ^^^^^^^^ Expected a tuple with 3 elements, found one with 2 elements
            ~~~~~~~~ The expression the tuple is assigned to has type `(Field,Field,Field)`
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_struct_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: Foo) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(moo2::Foo {});
                 ^^^^^^^^^^^^ Expected type Foo, found type Foo
                 ~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_generic_case() {
    let src = r#"
    pub struct Gen<T> {}

    mod moo {
        use super::Gen;

        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: Gen<Foo>) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(Gen::<moo2::Foo> {});
                 ^^^^^^^^^^^^^^^^^^^ Expected type Gen<Foo>, found type Gen<Foo>
                 ~~~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_tuple_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: (Foo, i32)) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo((moo2::Foo {}, 1));
                 ^^^^^^^^^^^^^^^^^ Expected type (Foo, i32), found type (Foo, Field)
                 ~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_array_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: [Foo; 1]) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo([moo2::Foo {}]);
                 ^^^^^^^^^^^^^^ Expected type [Foo; 1], found type [Foo; 1]
                 ~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_vector_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: [Foo]) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(@[moo2::Foo {}]);
                 ^^^^^^^^^^^^^^^ Expected type [Foo], found type [Foo]
                 ~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_reference_case() {
    let src = r#"
    mod moo {
        pub struct Foo {}
                   ~~~ Note: `moo::Foo` is defined in the current crate

        pub fn foo(_: &mut Foo) {}
    }

    mod moo2 {
        pub struct Foo {}
                   ~~~ Note: `moo2::Foo` is defined in the current crate
    }

    fn main() {
        moo::foo(&mut moo2::Foo {});
                 ^^^^^^^^^^^^^^^^^ Expected type &mut Foo, found type &mut Foo
                 ~~~~~~~~~~~~~~~~~ Note: `moo2::Foo` and `moo::Foo` have similar names, but are actually distinct types
    }
    "#;
    check_errors(src);
}

#[test]
fn type_mismatch_same_name_different_fully_qualified_name_cyclic_types() {
    let src = r#"
    pub struct Gen<T> {
               ^^^ Self-referential types are not supported
               ~~~ Note: `Gen` is defined in the current crate
        x: Gen<T>,
    }

    mod moo {
        pub struct Gen<T> {
                   ^^^ Self-referential types are not supported
                   ~~~ Note: `moo::Gen` is defined in the current crate
            x: Gen<T>,
        }
    }

    fn foo<T>(_: Gen<T>) {}

    fn main() {
        foo(moo::Gen::<i32> {})
            ^^^^^^^^^^^^^^^ missing field x in struct Gen
            ^^^^^^^^^^^^^^^^^^ Expected type Gen<_>, found type Gen<i32>
            ~~~~~~~~~~~~~~~~~~ Note: `moo::Gen` and `Gen` have similar names, but are actually distinct types
        ^^^ Type annotation needed
        ~~~ Could not determine the type of the generic argument `T` declared on the function `foo`
    }
    "#;
    check_errors(src);
}

#[test]
fn slice_used_in_assert_error_message() {
    let src = r#"
    fn main() {
        let a: [Field] = @[1, 2, 3];
        foo(a);
    }

    fn foo<T>(x: T) {
        assert(false, x);
                      ^ Invalid type [Field] used in the error message
                      ~ Error message fragments must be ABI compatible
    }
    "#;
    check_monomorphization_error(src);
}
