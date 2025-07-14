use crate::{assert_no_errors, check_errors};

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn identity_numeric_type_alias_works() {
    let src = r#"
    pub type Identity<let N: u32>: u32 = N;
    "#;
    assert_no_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}

#[named]
#[test]
fn disallows_composing_numeric_type_aliases() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = Double<Double<N>>;
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  Expected a numeric expression, but got `Double<Double<N>>`
    fn main() {
        let b: [u32; 12] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Quadruple::<N>] {
        let n = Double::<N>;    // To avoid the unused 'Double' error
        let mut a = [0;Quadruple::<N>];
        for i in 0..Quadruple::<N> {
            a[i] = i + n;
        }
        a
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn disallows_numeric_type_aliases_to_expression_with_alias() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = Double::<N>+Double::<N>;
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  Cannot use a type alias inside a type alias
    fn main() {
        let b: [u32; 12] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Quadruple::<N>] {
        let n = Double::<N>;    // To avoid the unused 'Double' error
        let mut a = [0;Quadruple::<N>];
        for i in 0..Quadruple::<N> {
            a[i] = i + n;
        }
        a
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn disallows_numeric_type_aliases_to_expression_with_alias_2() {
    let src = r#"
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = N*(Double::<N>+3);
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  Cannot use a type alias inside a type alias
    fn main() {
        let b: [u32; 12] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Quadruple::<N>] {
        let n = Double::<N>;    // To avoid the unused 'Double' error
        let mut a = [0;Quadruple::<N>];
        for i in 0..Quadruple::<N> {
            a[i] = i + n;
        }
        a
    }
    "#;
    check_errors!(src);
}

#[named]
#[test]
fn disallows_numeric_type_aliases_to_type() {
    let src = r#"
    type Foo: u32 = u32;
    ^^^^^^^^^^^^^^^^^^^  Expected a numeric expression, but got `u32`
    fn main(a: Foo) -> pub Foo {
        a
    }
    "#;
    check_errors!(src);
}

#[named]
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
    assert_no_errors!(src);
}
