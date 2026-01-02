// Regression for issue #7710 (https://github.com/noir-lang/noir/issues/7710)
unconstrained fn foo() -> [Field; 0] {
    println("foo");
    []
}

fn main() {
    // Safety: testing context
    let x = unsafe { foo() };

    let _ = x.as_vector().map(|_| {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        assert_constant(x);
        0
    });

    let _ = x.as_vector().mapi(|_, _| {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        assert_constant(x);
        0
    });

    x.as_vector().for_each(|_| {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        assert_constant(x);
    });

    x.as_vector().for_eachi(|_, _| {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        assert_constant(x);
    });

    for _ in x.as_vector() {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        std::assert_constant(x);
    }

    let _ = x.map(|_| {
        std::static_assert(false, "test");
        assert(false, "test");
        panic(f"test");
        assert_constant(x);
    });

    for _ in x {
        std::static_assert(false, "test");
        assert(false, "test");
        panic(f"test");
        assert_constant(x);
    }

    let empty_array: [Field; 0] = [];
    for _ in empty_array.as_vector() {
        std::static_assert(false, "test");
        assert(false, "test");
        panic(f"test");
        assert_constant(x);
    }

    for _ in 0..0 {
        std::static_assert(false, "test");
        assert(false, "test");
        panic(f"test");
        assert_constant(x);
    }
}
