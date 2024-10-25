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

// This is a regression test for https://github.com/noir-lang/noir/issues/6347
#[test]
#[should_panic = r#"ResolverError(Expected { span: Span(Span { start: ByteIndex(95), end: ByteIndex(98) }), expected: "type", got: "type alias" }"#]
fn allows_destructuring_a_type_alias_of_a_struct() {
    let src = r#"
    struct Foo {
        inner: Field
    }

    type Bar = Foo;

    fn main() {
        let Bar { inner } = Foo { inner: 42 };
        assert_eq(inner, 42);
    }
    "#;
    assert_no_errors(src);
}
