use crate::tests::{UnstableFeature, assert_no_errors, check_errors, check_errors_using_features};

#[test]
fn allows_usage_of_type_alias_as_argument_type() {
    let src = r#"
    type Foo = Field;

    fn accepts_a_foo(x: Foo) {
        assert_eq(x, 42);
    }

    fn main() {
        accepts_a_foo(42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_usage_of_type_alias_as_return_type() {
    let src = r#"
    type Foo = Field;

    fn returns_a_foo() -> Foo {
        42
    }

    fn main() {
        let _ = returns_a_foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn alias_in_let_pattern() {
    let src = r#"
        struct Foo<T> { x: T }

        type Bar<U> = Foo<U>;

        fn main() {
            let Bar { x } = Foo { x: [0] };
            // This is just to show the compiler knows this is an array.
            let _: [Field; 1] = x;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn double_alias_in_path() {
    let src = r#"
    struct Foo {}

    impl Foo {
        fn new() -> Self {
            Self {}
        }
    }

    type FooAlias1 = Foo;
    type FooAlias2 = FooAlias1;

    fn main() {
        let _ = FooAlias2::new();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn double_generic_alias_in_path() {
    let src = r#"
    struct Foo<T> {}

    impl<T> Foo<T> {
        fn new() -> Self {
            Self {}
        }
    }

    type FooAlias1 = Foo<i32>;
    type FooAlias2 = FooAlias1;

    fn main() {
        let _ = FooAlias2::new();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deny_cyclic_type_aliases() {
    let src = r#"
        type A = B;
        type B = A;
             ^ Dependency cycle found
             ~ 'B' recursively depends on itself: B -> A -> B
    "#;
    check_errors(src);
}

#[test]
fn cyclic_type_alias_usage_does_not_stack_overflow() {
    let src = r#"
        type A = B;
        type B = A;
             ^ Dependency cycle found
             ~ 'B' recursively depends on itself: B -> A -> B
        fn main() {
            let _ = A::foo();
                    ^ Could not resolve 'A' in path
        }
    "#;
    check_errors(src);
}

#[test]
fn ensure_nested_type_aliases_type_check() {
    let src = r#"
        type A = B;
        type B = u8;
        fn main() {
            let _a: A = 0 as u16;
                        ^^^^^^^^ Expected type A, found type u16
        }
    "#;
    check_errors(src);
}

#[test]
fn type_aliases_in_entry_point() {
    let src = r#"
        type Foo = u8;
        fn main(_x: Foo) {}
    "#;
    assert_no_errors(src);
}

// Regression for #4545
#[test]
fn array_type_aliases_in_main() {
    let src = r#"
        type Outer<let N: u32> = [u8; N];
        fn main(_arg: Outer<1>) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn identity_numeric_type_alias_works() {
    let src = r#"
    pub type Identity<let N: u32>: u32 = N;
    "#;
    assert_no_errors(src);
}

#[test]
fn self_referring_type_alias_is_not_allowed() {
    let src = r#"
        pub type X = X;

        fn main() {
            let _: X = 1;
                   ^ Binding `X` here to the `_` inside would create a cyclic type
                   ~ Cyclic types have unlimited size and are prohibited in Noir
        }
      "#;
    check_errors(src);
}

#[test]
fn type_alias_to_numeric_generic() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    fn main() {
        let b: [u32; 6] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Double::<N>] {
        let mut a = [0;Double::<N>];
        for i in 0..Double::<N> {
            a[i] = i;
        }
        a
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn disallows_composing_numeric_type_aliases_as_type_syntax() {
    // Double<Double<N>> uses type syntax (Named with generics), not expression syntax.
    // try_into_expression rejects Named with non-empty generics.
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = Double<Double<N>>;
                                      ^^^^^^^^^^^^^^^^^^ Expected a numeric expression, but got `Double<Double<N>>`
    fn main() {
        let _ = Double::<1>;
        let _ = Quadruple::<1>;
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_composing_numeric_type_aliases_in_expression() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = Double::<N>+Double::<N>;
    fn main() {
        let b: [u32; 12] = foo::<3>();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Quadruple::<N>] {
        let mut a = [0;Quadruple::<N>];
        for i in 0..Quadruple::<N> {
            a[i] = i;
        }
        a
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_composing_numeric_type_aliases_in_expression_2() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Mixed<let N: u32>: u32 = N*(Double::<N>+3);

    fn main() {
        let b: [u32; 14] = foo::<2>();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Mixed::<N>] {
        let mut a = [0;Mixed::<N>];
        for i in 0..Mixed::<N> {
            a[i] = i;
        }
        a
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn disallows_numeric_type_aliases_to_type() {
    let src = r#"
    type Foo: u32 = u32;
                    ^^^ Type provided when a numeric generic was expected
                    ~~~ the numeric generic is not of type `u32`

    fn main(a: Foo) -> pub Foo {
        a
    }
    "#;
    check_errors(src);
}

#[test]
fn type_alias_to_numeric_as_generic() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;

    pub struct Foo<T, let N: u32> {
        a: T,
        b: [Field; N],
    }
    fn main(x: Field) {
        let a = foo::<4>(x);
        assert(a.a == x);
    }
    fn foo<let N:u32>(x: Field) -> Foo<Field, Double<N>> {
        Foo {
            a: x,
            b: [1; Double::<N>]
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn self_referring_type_alias_with_generics_is_allowed() {
    let src = r#"
        type Id<T> = T;

        fn main() {
            let _: Id<Id<Field>> = 1;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_type_alias_in_method_call() {
    let src = r#"
        pub struct Foo {
        }

        impl Foo {
            fn new() -> Self {
                Foo {}
            }
        }

        type Bar = Foo;

        fn foo() -> Foo {
            Bar::new()
        }

        fn main() {
            let _ = foo();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_type_alias_to_generic_concrete_type_in_method_call() {
    let src = r#"
        pub struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T> {
            fn new(x: T) -> Self {
                Foo { x }
            }
        }

        type Bar = Foo<i32>;

        fn foo() -> Bar {
            Bar::new(1)
        }

        fn main() {
            let _ = foo();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn incorrect_generic_count_on_type_alias() {
    let src = r#"
    pub struct Foo {}
    pub type Bar = Foo<i32>;
                   ^^^ Foo expects 0 generics but 1 was given
    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn call_function_alias_type() {
    let src = r#"
    type Alias<Env> = fn[Env](Field) -> Field;

    fn main() {
        call_fn(|x| x + 1);
    }

    fn call_fn<Env>(f: Alias<Env>) {
        assert_eq(f(0), 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10415() {
    let src = r#"
    type Nothing = ();

    fn main() -> Nothing {}
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10415_without_alias() {
    let src = r#"
    fn main() -> () {}
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10429() {
    let src = r#"
    struct Struct {}

    type Alias = Struct;

    impl Alias {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10429_with_trait() {
    let src = r#"
    struct Struct {}

    type Alias = Struct;

    trait Foo {
        fn foo() -> Self;
    }

    impl Foo for Alias {
        fn foo() -> Self {
            Struct {}
        }
    }

    fn main() {
        let _alias: Alias = <Alias as Foo>::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10352_parameter() {
    let src = r#"
    type Alias = Alias;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_tuple() {
    let src = r#"
    type Alias = (Alias,);

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_struct() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    type Alias = Foo<Alias>;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_enum() {
    let src = r#"
    enum Foo<T> {
        Bar(T),
        Baz,
    }

    type Alias = Foo<Alias>;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_array() {
    let src = r#"
    type Alias = [Alias; 3];

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_slice() {
    let src = r#"
    type Alias = [Alias];

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_trait_as_type() {
    let src = r#"
    struct Foo {
        x: impl Bar,
                ^^^ `impl Trait` is not allowed in struct field types
                ~~~ Use a generic type parameter instead
    }

    trait Bar {
        fn bar(self);
    }
    impl Bar for Foo {
        fn bar(self) {
            self.x.bar();
        }
    }

    fn main(_a: Foo) {}
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn regression_10352_string() {
    let src = r#"
    type Alias = str<Alias>;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_format_string_len() {
    let src = r#"
    type Alias = fmtstr<Alias, ()>;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_format_string_env() {
    let src = r#"
    type Alias = fmtstr<0, (Alias,)>;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_function_parameter() {
    let src = r#"
    type Alias = fn(Alias);

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_function_return() {
    let src = r#"
    type Alias = fn() -> Alias;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_function_env() {
    let src = r#"
    type Alias = fn[(Alias,)]();

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn regression_10352_immutable_reference() {
    let src = r#"
    type Alias = &Alias;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors_using_features(src, &[UnstableFeature::Ownership]);
}

#[test]
fn regression_10352_mutable_reference() {
    let src = r#"
    type Alias = &mut Alias;

    fn main(_: Alias) {}
               ^^^^^ Binding `Alias` here to the `_` inside would create a cyclic type
               ~~~~~ Cyclic types have unlimited size and are prohibited in Noir
    "#;
    check_errors(src);
}

#[test]
fn ensure_repeated_aliases_in_tuples_are_not_detected_as_cyclic_aliases() {
    let src = r#"
    type K = Field;
    type V = Field;

    fn field_lt(_x: Field, _y: Field) -> bool { true }

    pub global KV_CMP: fn((K, V), (K, V)) -> bool = |a: (K, V), b: (K, V)| field_lt(a.0, b.0);

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn ensure_repeated_aliases_in_arrays_are_not_detected_as_cyclic_aliases() {
    let src = r#"
    pub type TReturnElem = [Field; 3];
    pub type TReturn = [TReturnElem; 2];

    pub fn t_return_elem() -> TReturnElem {
        [0; 3]
    }

    pub fn t_return() -> TReturn {
        [t_return_elem(); 2]
    }

    pub unconstrained fn two_nested_return_unconstrained() -> (Field, TReturn, Field, TReturn) {
        (0, t_return(), 0, t_return())
    }

    pub unconstrained fn foo_return_unconstrained() -> (Field, TReturn, TestTypeFoo) {
        (0, t_return(), test_type_foo())
    }

    pub struct TestTypeFoo {
        a: Field,
        b: [[[Field; 3]; 4]; 2],
        c: [TReturnElem; 2],
        d: TReturnElem,
    }

    pub fn test_type_foo() -> TestTypeFoo {
        TestTypeFoo {
            a: 0,
            b: [[[0; 3]; 4]; 2],
            c: [t_return_elem(); 2],
            d: t_return_elem(),
        }
    }

    pub unconstrained fn complex_struct_return() {
        let _: (Field, [[Field; 3]; 2], TestTypeFoo) = foo_return_unconstrained();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10763_mutable() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    type Bar = &mut ();

    impl Foo for Bar {
                 ^^^ Trait impls are not allowed on aliases to reference types
                 ~~~ Try using a struct or enum type here instead

        fn foo(self) { }
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10756() {
    let src = r#"
    pub type Foo = 0;
                   ^ type expression is not allowed for type aliases (Is this a numeric type alias? If so, the numeric type must be specified with `: <type>`

    fn main() {
        let _: Foo = std::mem::zeroed();
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10763_immutable() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    type Bar = &();

    impl Foo for Bar {
                 ^^^ Trait impls are not allowed on aliases to reference types
                 ~~~ Try using a struct or enum type here instead
        fn foo(self) { }
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::Ownership]);
}

#[test]
fn signed_numeric_type_alias_with_negative_operand() {
    // Regression test for https://github.com/noir-lang/noir/issues/10969
    // Numeric type alias expressions should use their declared type.
    // The expression `0 % (-1)` must be elaborated as a i32.
    // An unsigned type would cause an "attempt to subtract with overflow" errors.
    let src = r#"
    pub type X: i32 = 0i32 % -1i32;

    fn main() {
        let _: i32 = X;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_10971() {
    // Regression test for https://github.com/noir-lang/noir/issues/10971
    let src = r#"
    pub type X: u8 = 257u8;
    ^^^^^^^^^^^^^^^^^^^^^^ The value `257` cannot fit into `u8` which has range `0..=255`
                     ^^^^^ The value `257` cannot fit into `u8` which has range `0..=255`

    fn main() {
        let _ = X;
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_trait_as_type_with_empty_trait() {
    let src = r#"
    trait Foo { }

    type Bar = impl Foo;
                    ^^^ `impl Trait` is not allowed in type alias definitions
                    ~~~ Use a generic type parameter instead

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn regression_10764_undefined_generic_with_empty_trait() {
    let src = r#"
    trait Foo { }

    type Bar = N;
               ^ Could not resolve 'N' in path

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_underscore_with_empty_trait() {
    let src = r#"
    trait Foo { }

    type Bar = _;
               ^ The placeholder `_` is not allowed in type alias definitions

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_trait_as_type() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    type Bar = impl Foo;
                    ^^^ `impl Trait` is not allowed in type alias definitions
                    ~~~ Use a generic type parameter instead

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
        fn foo(self) { }
           ^^^ Cannot define a method on values of type `Bar`
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn regression_10764_undefined_generic() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    type Bar = N;
               ^ Could not resolve 'N' in path

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
        fn foo(self) { }
           ^^^ Cannot define a method on values of type `Bar`
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_underscore() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    type Bar = _;
               ^ The placeholder `_` is not allowed in type alias definitions

    impl Foo for Bar {
                 ^^^ Cannot define a trait impl on values of type `Bar`
        fn foo(self) { }
           ^^^ Cannot define a method on values of type `Bar`
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_trait_as_type_impl() {
    let src = r#"
    trait Foo {
          ^^^ unused trait Foo
          ~~~ unused trait
        fn foo(self);
    }

    type Bar = impl Foo;
                    ^^^ `impl Trait` is not allowed in type alias definitions
                    ~~~ Use a generic type parameter instead

    impl Bar {
         ^^^ Non-enum, non-struct type used in impl
         ~~~ Only enum and struct types may have implementation methods
        fn foo() { }
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn regression_10764_undefined_generic_impl() {
    let src = r#"
    type Foo = N;
               ^ Could not resolve 'N' in path

    impl Foo {
         ^^^ Non-enum, non-struct type used in impl
         ~~~ Only enum and struct types may have implementation methods
        fn foo() { }
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10764_underscore_impl() {
    let src = r#"
    type Foo = _;
               ^ The placeholder `_` is not allowed in type alias definitions

    impl Foo {
         ^^^ Non-enum, non-struct type used in impl
         ~~~ Only enum and struct types may have implementation methods
        fn foo() { }
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_assign_to_numeric_type_alias() {
    let src = r#"
    type N: u32 = 1;

    fn main() {
        N = 2;
        ^ expected value, found type alias `N`
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_type_alias_referencing_non_numeric_type_alias() {
    // A numeric type alias referencing a non-numeric type alias should error
    // because `Two` is not a numeric alias.
    let src = r#"
    type One: u32 = Two;
                    ^^^^ Numeric type alias expression must only reference generic parameters and constants
    type Two = u32;

    fn main(a: One) -> pub Two {
        a
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_alias_turbofish_resolves_correctly() {
    // Turbofish on a numeric type alias should resolve to the correct value,
    // not to a global that happens to share the same name.
    // The alias generic should not leak and shadow the global after usage.
    // This only makes compile-time checks, for verifying the runtime values' correctness the same logic can be found
    // under the `test_programs/execution_success/numeric_type_alias` integration test.
    let src = r#"
    global N: u32 = 100;

    type Alias<let N: u32>: u32 = N;
    type Double<let N: u32>: u32 = N * 2;
    type X: u32 = 42;

    fn main() {
        // Turbofish resolves to the supplied value, not the global
        let a: u32 = Alias::<1>;
        assert(a == 1);
        // Expression body works with turbofish
        let b: u32 = Double::<3>;
        assert(b == 6);
        // Numeric alias w/o generics works without turbofish
        let c: u32 = X;
        assert(c == 42);
        // N should still refer to the global, not the alias generic
        assert(N == 100);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_alias_in_range_expression() {
    // Numeric type alias should work in range expressions (value position)
    let src = r#"
    type Size<let N: u32, let M: u32>: u32 = N * M;

    fn foo<let N: u32, let M: u32>() -> Field {
        let mut s: Field = 0;
        for i in 0..Size::<N, M> {
            s += i as Field;
        }
        s
    }

    fn main() {
        let result = foo::<2, 3>();
        // sum of 0..6 = 0+1+2+3+4+5 = 15
        assert(result == 15);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_type_alias_with_global_constants() {
    // Numeric type aliases should be able to reference global constants
    let src = r#"
    global One: u32 = 1;
    type Two: u32 = One + One;

    fn main() {
        let a: u32 = Two;
        assert(a == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_type_alias_with_other_numeric_alias() {
    // Numeric type aliases should be able to reference other numeric type aliases
    // (without generics)
    let src = r#"
    type One: u32 = 1;
    type Two: u32 = One + One;

    fn main() {
        let a: u32 = Two;
        assert(a == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_type_alias_referencing_struct() {
    // A numeric type alias referencing a struct should error.
    // The struct resolves as a named type, but fails kind-checking since it's not numeric.
    let src = r#"
    struct Foo { x: u32 }
    type Bad: u32 = Foo;
                    ^^^ Type provided when a numeric generic was expected
                    ~~~ the numeric generic is not of type `u32`

    fn main(a: Bad) -> pub Foo {
        Foo { x: a }
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_type_alias_referencing_tuple_type() {
    // A numeric type alias referencing a tuple type should error.
    // Tuple types cannot be converted to expressions via try_into_expression.
    let src = r#"
    type Bad: u32 = (Field, Field);
                    ^^^^^^^^^^^^^^^ Expected a numeric expression, but got `(Field, Field)`

    fn main(a: Bad) -> pub Bad {
        a
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_type_alias_referencing_array_type() {
    // A numeric type alias referencing an array type should error.
    // Array types cannot be converted to expressions via try_into_expression.
    let src = r#"
    type Bad: u32 = [Field; 3];
                    ^^^^^^^^^^^ Expected a numeric expression, but got `[Field; 3]`

    fn main(a: Bad) -> pub Bad {
        a
    }
    "#;
    check_errors(src);
}

#[test]
fn numeric_type_alias_referencing_function() {
    // A numeric type alias referencing a function name should error.
    // `foo` resolves as a variable expression, but the type resolver catches it
    // as a function rather than a type.
    let src = r#"
    fn foo() -> u32 { 1 }
    type Bad: u32 = foo;
                    ^^^ expected type, found function `foo`

    fn main(a: Bad) -> pub Bad {
        a
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_using_comptime_type_in_non_comptime_type_alias() {
    let src = r#"
    pub type Alias = Quoted;
                     ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime type alias
    "#;
    check_errors(src);
}

/// Regression test: a type alias and a global with the same name
#[test]
fn type_alias_takes_priority_over_global_with_same_name() {
    let src = r#"
        global Foo: u32 = 10;

        type Foo = u32;

        fn main() {
            let x: Foo = 20;
            assert(x == 20);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn type_alias_as_closure_environment() {
    let src = r#"
    type Env = (u32,);

    pub fn foo(_x: fn[Env](Field) -> Field) {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

/// Regression test: define_type_alias did not reset `current_item` after finishing,
/// which can leak into subsequent elaboration phases.
#[test]
fn no_false_cycle_from_stale_current_item_after_type_alias() {
    // `A` depends on `B` (real dependency).
    // After the type-alias loop, `current_item` is left as `Alias(B)`.
    // When `collect_traits` resolves the supertrait `Dummy<A>`, it should not
    // record a dependency from `B` to `A` (which would create a false A↔B cycle).
    let src = r#"
        type A = B;
        type B = Field;

        trait Dummy<T> {}
        trait Foo: Dummy<A> {}

        fn main(_x: A) where A: Foo {}
    "#;
    assert_no_errors(src);
}
