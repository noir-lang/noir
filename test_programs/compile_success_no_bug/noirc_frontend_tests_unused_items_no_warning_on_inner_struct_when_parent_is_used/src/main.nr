
    struct Bar {
        inner: [Field; 3],
    }

    struct Foo {
        a: Field,
        bar: Bar,
    }

    fn main(foos: [Foo; 1]) {
        assert_eq(foos[0].a, 10);
    }
    