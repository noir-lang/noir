use crate::tests::{assert_no_errors, check_errors, check_errors_using_features, UnstableFeature};

#[test]
fn numeric_generic_in_function_signature() {
    let src = r#"
    pub fn foo<let N: u32>(arr: [Field; N]) -> [Field; N] { arr }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_as_struct_field_type_fails() {
    let src = r#"
    pub struct Foo<let N: u32> {
        a: Field,
        b: N,
           ^ Expected type, found numeric generic
           ~ not a type
    }
    "#;
    check_errors(src);
}

#[test]
fn normal_generic_as_array_length() {
    // TODO: improve error location, should be just on N
    let src = r#"
    pub struct Foo<N> {
        a: Field,
        b: [Field; N],
           ^^^^^^^^^^ Type provided when a numeric generic was expected
           ~~~~~~~~~~ the numeric generic is not of type `u32`
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_array_len() {
    let src = r#"
        struct Array<T, let N: u32> {
            inner: [T; N],
        }

        impl<T, let N: u32> Array<T, N> {
            pub fn len(self) -> u32 {
                       ^^^^ unused variable self
                       ~~~~ unused variable
                N as u32
            }
        }

        fn main(xs: [Field; 2]) {
            let ys = Array {
                inner: xs,
            };
            assert(ys.len() == 2);
        }
    "#;
    check_errors(src);
}

// Regression for #2540
#[test]
fn for_loop_over_array() {
    let src = r#"
        fn hello<let N: u32>(_array: [u1; N]) {
            for _ in 0..N {}
        }

        fn main() {
            let array: [u1; 2] = [0, 1];
            hello(array);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_as_param_type() {
    let src = r#"
    pub fn foo<let I: u32>(x: I) -> I {
                                    ^ Expected type, found numeric generic
                                    ~ not a type
                              ^ Expected type, found numeric generic
                              ~ not a type
                                    

        let _q: I = 5;
                ^ Expected type, found numeric generic
                ~ not a type
        x
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_as_unused_param_type() {
    let src = r#"
    pub fn foo<let I: u32>(_x: I) { }
                               ^ Expected type, found numeric generic
                               ~ not a type
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_as_unused_trait_fn_param_type() {
    let src = r#"
    trait Foo {
          ^^^ unused trait Foo
          ~~~ unused trait
        fn foo<let I: u32>(_x: I) { }
                               ^ Expected type, found numeric generic
                               ~ not a type
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_as_return_type() {
    let src = r#"
    // std::mem::zeroed() without stdlib
    trait Zeroed {
        fn zeroed<T>(self) -> T;
    }

    fn foo<T, let I: Field>(x: T) -> I where T: Zeroed {
                                     ^ Expected type, found numeric generic
                                     ~ not a type
       ^^^ unused function foo
       ~~~ unused function
        x.zeroed()
        ^^^^^^^^ Type annotation needed
        ~~~~~~~~ Could not determine the type of the generic argument `T` declared on the function `zeroed`
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_used_in_nested_type_fails() {
    let src = r#"
    pub struct Foo<let N: u32> {
        a: Field,
        b: Bar<N>,
    }
    pub struct Bar<let N: u32> {
        inner: N
               ^ Expected type, found numeric generic
               ~ not a type
    }
    "#;
    check_errors(src);
}

#[test]
fn normal_generic_used_in_nested_array_length_fail() {
    let src = r#"
    pub struct Foo<N> {
        a: Field,
        b: Bar<N>,
               ^ Type provided when a numeric generic was expected
               ~ the numeric generic is not of type `u32`
    }
    pub struct Bar<let N: u32> {
        inner: [Field; N]
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_used_in_nested_type_pass() {
    // The order of these structs should not be changed to make sure
    // that we are accurately resolving all struct generics before struct fields
    let src = r#"
    pub struct NestedNumeric<let N: u32> {
        a: Field,
        b: InnerNumeric<N>
    }
    pub struct InnerNumeric<let N: u32> {
        inner: [u64; N],
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_trait() {
    // We want to make sure that `N` in `impl<let N: u32, T> Deserialize<N, T>` does
    // not trigger `expected type, found numeric generic parameter N` as the trait
    // does in fact expect a numeric generic.
    let src = r#"
    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }

    impl<let N: u32, T> Deserialize<N, T> for MyType<T> {
        fn deserialize(fields: [Field; N], other: T) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: other }
        }
    }

    trait Deserialize<let N: u32, T> {
        fn deserialize(fields: [Field; N], other: T) -> Self;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_in_trait_impl_with_extra_impl_generics() {
    let src = r#"
    trait Default2 {
        fn default2() -> Self;
    }

    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }

    // Make sure that `T` is placed before `N` as we want to test that the order of the generics is correctly maintained.
    // `N` is used first in the trait impl generics (`Deserialize<N> for MyType<T>`).
    // We want to make sure that the compiler correctly accounts for that `N` has a numeric kind
    // while `T` has a normal kind.
    impl<T, let N: u32> Deserialize<N> for MyType<T> where T: Default2 {
        fn deserialize(fields: [Field; N]) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: T::default2() }
        }
    }

    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_where_clause() {
    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    pub fn read<T, let N: u32>() -> T where T: Deserialize<N> {
        let mut fields: [Field; N] = [0; N];
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_turbofish() {
    let src = r#"
    pub fn double<let N: u32>() -> u32 {
        // Used as an expression
        N * 2
    }

    pub fn double_numeric_generics_test() {
        // Example usage of a numeric generic arguments.
        assert(double::<9>() == 18);
        assert(double::<7 + 8>() == 30);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_u16_array_size() {
    // TODO: improve the error location
    let src = r#"
    fn len<let N: u32>(_arr: [Field; N]) -> u32 {
        N
    }

    pub fn foo<let N: u16>() -> u32 {
        let fields: [Field; N] = [0; N];
                                     ^ The numeric generic is not of type `u32`
                                     ~ expected `u32`, found `u16`
                    ^^^^^^^^^^ The numeric generic is not of type `u32`
                    ~~~~~~~~~~ expected `u32`, found `u16`
        len(fields)
        ^^^ Type annotation needed
        ~~~ Could not determine the value of the generic argument `N` declared on the function `len`
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_field_larger_than_u32() {
    let src = r#"
        global A: Field = 4294967297;

        fn foo<let A: Field>() { }

        fn main() {
            let _ = foo::<A>();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_field_arithmetic_larger_than_u32() {
    let src = r#"
        struct Foo<let F: Field> {}

        fn size<let F: Field>(_x: Foo<F>) -> Field {
            F
        }

        // 2^32 - 1
        global A: Field = 4294967295;

        fn foo<let A: Field>() -> Foo<A + A> {
            Foo {}
        }

        fn main() {
            let _ = size(foo::<A>());
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn constant_used_with_numeric_generic() {
    let src = r#"
    struct ValueNote {
        value: Field,
    }

    trait Serialize<let N: u32> {
        fn serialize(self) -> [Field; N];
    }

    impl Serialize<1> for ValueNote {
        fn serialize(self) -> [Field; 1] {
            [self.value]
        }
    }

    fn main() {
        let _ = ValueNote { value: 1 }; // silence ValueNote never constructed warning
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn normal_generic_used_when_numeric_expected_in_where_clause() {
    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    pub fn read<T, N>() -> T where T: Deserialize<N> {
                                                  ^ Type provided when a numeric generic was expected
                                                  ~ the numeric generic is not of type `u32`
        T::deserialize([0, 1])
    }
    "#;
    check_errors(src);

    // TODO: improve the error location for the array (should be on N)
    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    pub fn read<T, N>() -> T where T: Deserialize<N> {
                                                  ^ Type provided when a numeric generic was expected
                                                  ~ the numeric generic is not of type `u32`
        let mut fields: [Field; N] = [0; N];
                                         ^ Type provided when a numeric generic was expected
                                         ~ the numeric generic is not of type `u32`
                        ^^^^^^^^^^ Type provided when a numeric generic was expected
                        ~~~~~~~~~~ the numeric generic is not of type `u32`
        for i in 0..N {
                    ^ cannot find `N` in this scope
                    ~ not found in this scope
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generics_type_kind_mismatch() {
    let src = r#"
    fn foo<let N: u32>() -> u16 {
        N as u16
    }

    global J: u16 = 10;

    fn bar<let N: u16>() -> u16 {
        foo::<J>()
              ^ The numeric generic is not of type `u32` 
              ~ expected `u32`, found `u16`
    }

    global M: u16 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generics_value_kind_mismatch_u32_u64() {
    let src = r#"
    struct BoundedVec<T, let MaxLen: u32> {
        storage: [T; MaxLen],
        // can't be compared to MaxLen: u32
        // can't be used to index self.storage
        len: u64,
    }

    impl<T, let MaxLen: u32> BoundedVec<T, MaxLen> {
        pub fn extend_from_bounded_vec<let Len: u32>(&mut self, _vec: BoundedVec<T, Len>) {
            // We do this to avoid an unused variable warning on `self`
            let _ = self.len;
            for _ in 0..Len { }
        }

        pub fn push(&mut self, elem: T) {
            assert(self.len < MaxLen, "push out of bounds");
                   ^^^^^^^^^^^^^^^^^ Integers must have the same bit width LHS is 64, RHS is 32
            self.storage[self.len] = elem;
                         ^^^^^^^^ Indexing arrays and vectors must be done with `u32`, not `u64`
            self.len += 1;
        }
    }

    fn main() {
        let _ = BoundedVec { storage: [1], len: 1 }; // silence never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn use_non_u32_generic_in_struct() {
    let src = r#"
        struct S<let N: u8> {}

        fn main() {
            let _: S<3> = S {};
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_numeric_generic_in_trait_method() {
    let src = r#"
        trait Foo  {
            fn foo<let N: u32>(self, x: [u8; N]) -> Self;
        }

        struct Bar;

        impl Foo for Bar {
            fn foo<let N: u32>(self, _x: [u8; N]) -> Self {
                self
            }
        }

        fn main() {
            let bytes: [u8; 3] = [1,2,3];
            let _ = Bar{}.foo(bytes);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn struct_numeric_generic_in_function() {
    let src = r#"
    struct Foo {
        inner: u64
    }

    pub fn bar<let N: Foo>() {
                      ^^^ N has a type of Foo. The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`.
                      ~~~ Unsupported numeric generic type
        let _ = Foo { inner: 1 }; // silence Foo never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_numeric_generic_in_struct() {
    let src = r#"
    pub struct Foo {
        inner: u64
    }

    pub struct Bar<let N: Foo> { }
                          ^^^ N has a type of Foo. The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`.
                          ~~~ Unsupported numeric generic type
    "#;
    check_errors(src);
}

#[test]
fn bool_numeric_generic() {
    let src = r#"
    pub fn read<let N: bool>() -> Field {
                       ^^^^ N has a type of bool. The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`.
                       ~~~~ Unsupported numeric generic type
        if N {
            0
        } else {
            1
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_generic_binary_operation_type_mismatch() {
    let src = r#"
    pub fn foo<let N: Field>() -> bool {
        let mut check: bool = true;
        check = N;
                ^ Cannot assign an expression of type Field to a value of type bool
        check
    }
    "#;
    check_errors(src);
}

#[test]
fn bool_generic_as_loop_bound() {
    let src = r#"
    pub fn read<let N: bool>() {
                       ^^^^ N has a type of bool. The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`.
                       ~~~~ Unsupported numeric generic type
        let mut fields = [0; N];
                             ^ The numeric generic is not of type `u32`
                             ~ expected `u32`, found `bool`
        for i in 0..N { 
                    ^ Expected type Field, found type bool
            fields[i] = i + 1;
        }
        assert(fields[0] == 1);
    }
    "#;
    check_errors(src);
}

/// Regression for CI issue in https://github.com/noir-lang/noir/pull/10330
#[test]
fn integer_with_suffix_used_as_type_in_quote() {
    let src = "
        #[make_bar]
        fn main() {
            bar([1, 2]);
        }

        comptime fn make_bar(_f: FunctionDefinition) -> Quoted {
            let n = 2u32;
            quote {
                fn bar(_array: [Field; $n]) {}
            }
        }
    ";
    assert_no_errors(src);
}

/// Regression for https://github.com/noir-lang/noir/pull/10330#issuecomment-3499399843
#[test]
fn integer_with_suffix_used_as_tuple_index() {
    let src = "
        fn main() {
            macro!();
        }

        comptime fn macro() -> Quoted {
            let n = 0u32;
            quote {
                let tuple = (0u8, 1u16, 2i8);
                assert_eq(tuple.$n, 0);
            }
        }
    ";
    assert_no_errors(src);
}

// Regression for https://github.com/noir-lang/noir/issues/10711
#[test]
fn no_panic_on_numeric_generic_parse_error() {
    let src = "
        fn foo<let N: >() {
                      ^ Expected a type but found '>'
            let _ = N;
        }

        fn main() { foo::<3>(); }
    ";
    check_errors(src);
}

#[test]
fn regression_10431_struct_impl() {
    let src = r#"
    pub struct Foo<T> {
        x: T,
    }

    impl Foo<1> {}
             ^ Expected type, found numeric generic
             ~ not a type

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_10431_enum_impl() {
    let src = r#"
    pub enum Foo<T> {
        Bar(T),
    }

    impl Foo<1> {}
             ^ Expected type, found numeric generic
             ~ not a type

    fn main() {}
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn regression_10431_struct_function_parameter() {
    let src = r#"
    pub struct Foo<T> {
        x: T,
    }

    pub fn foo(_x: Foo<1>) {}
                       ^ Expected type, found numeric generic
                       ~ not a type

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_10431_enum_function_parameter() {
    let src = r#"
    pub enum Foo<T> {
        Bar(T),
    }

    pub fn foo(_x: Foo<1>) {}
                       ^ Expected type, found numeric generic
                       ~ not a type

    fn main() {}
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn regression_10431_struct_function_return() {
    let src = r#"
    pub struct Foo<T> { }

    pub fn foo() -> Foo<1> {
                        ^ Expected type, found numeric generic
                        ~ not a type
        Foo {}
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_10431_enum_function_return() {
    let src = r#"
    pub enum Foo<T> {
        Bar(T),
        Baz,
    }

    pub fn foo() -> Foo<1> {
                        ^ Expected type, found numeric generic
                        ~ not a type
        Foo::Baz
    }

    fn main() {}
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn regression_10431_struct_trait_impl() {
    let src = r#"
    pub struct Foo<T> {
        x: T,
    }

    trait Bar {}
    impl Bar for Foo<1> {}
                     ^ Expected type, found numeric generic
                     ~ not a type

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_10431_enum_trait_impl() {
    let src = r#"
    pub enum Foo<T> {
        Bar(T),
    }

    trait Bar {}
    impl Bar for Foo<1> {}
                     ^ Expected type, found numeric generic
                     ~ not a type

    fn main() {}
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn regression_10431_enum() {
    let src = r#"
    pub enum Foo<T> {
        Bar(T),
    }

    impl Foo<1> {}
             ^ Expected type, found numeric generic
             ~ not a type

    fn main() {}
    "#;
    let features = vec![UnstableFeature::Enums];
    check_errors_using_features(src, &features);
}

#[test]
fn regression_10431() {
    let src = r#"
    pub struct Foo<T> {
        x: T,
    }

    impl Foo<1> {}
             ^ Expected type, found numeric generic
             ~ not a type

    fn main() {}
    "#;
    check_errors(src);
}
