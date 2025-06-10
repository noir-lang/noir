#[derive(Eq)]
struct Foo {}

fn main() {
    let (a, b, .., x, y, z) = (1, true, Foo {}, [10], [true], ["hello"]);
    assert_eq(a, 1);
    assert_eq(b, true);
    assert_eq(x, [10]);
    assert_eq(y, [true]);
    assert_eq(z, ["hello"]);

    comptime {
        let (a, b, .., x, y, z) = (1, true, Foo {}, [10], [true], ["hello"]);
        assert_eq(a, 1);
        assert_eq(b, true);
        assert_eq(x, [10]);
        assert_eq(y, [true]);
        assert_eq(z, ["hello"]);
    }

    foo((1, 2, 3, 4, "hello", [true]));

    comptime {
        foo((1, 2, 3, 4, "hello", [true]));
    }

    let (.., a, b) = (1, true, Foo {}, "hello");
    assert_eq(a, Foo {});
    assert_eq(b, "hello");

    comptime {
        let (.., a, b) = (1, true, Foo {}, "hello");
        assert_eq(a, Foo {});
        assert_eq(b, "hello");
    }

    bar((1, true, Foo {}, "hello"));

    comptime {
        bar((1, true, Foo {}, "hello"));
    }

    let (a, b, ..) = (1, true, Foo {}, "hello");
    assert_eq(a, 1);
    assert_eq(b, true);

    comptime {
        let (a, b, ..) = (1, true, Foo {}, "hello");
        assert_eq(a, 1);
        assert_eq(b, true);
    }

    baz((1, true, Foo {}, "hello"));

    comptime {
        baz((1, true, Foo {}, "hello"));
    }

    let (x, y, ..) = (1, true);
    assert_eq(x, 1);
    assert_eq(y, true);

    comptime {
        let (x, y, ..) = (1, true);
        assert_eq(x, 1);
        assert_eq(y, true);
    }

    let (.., x, y) = (1, true);
    assert_eq(x, 1);
    assert_eq(y, true);

    comptime {
        let (.., x, y) = (1, true);
        assert_eq(x, 1);
        assert_eq(y, true);
    }

    let (..) = (1, 2, 3);
    comptime {
        let (..) = (1, 2, 3);
    }
}

fn foo((x, .., y, z): (i32, Field, i64, u8, str<5>, [bool; 1])) {
    assert_eq(x, 1);
    assert_eq(y, "hello");
    assert_eq(z, [true]);
}

fn bar((.., a, b): (i32, bool, Foo, str<5>)) {
    assert_eq(a, Foo {});
    assert_eq(b, "hello");
}

fn baz((a, b, ..): (i32, bool, Foo, str<5>)) {
    assert_eq(a, 1);
    assert_eq(b, true);
}
