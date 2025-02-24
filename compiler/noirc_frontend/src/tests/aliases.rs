use super::assert_no_errors;

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
    "#;
    assert_no_errors(src);
}
