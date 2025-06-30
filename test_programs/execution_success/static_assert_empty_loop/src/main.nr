unconstrained fn foo() -> [Field; 0] {
    println("foo");
    []
}

fn main() {
    // safety: no
    let x = unsafe { foo() };

    let _ = x.as_slice().map(|_| {
        assert(false, "test");
        panic(f"test");

        // these fail:
        std::static_assert(false, "test");
        assert_constant(x);
        0
    });

    for _ in x.as_slice() {
        assert(false, "test");
        panic(f"test");

        std::static_assert(false, "test");
        std::assert_constant(x);
    }

    // these all work:
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
    for _ in empty_array.as_slice() {
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
