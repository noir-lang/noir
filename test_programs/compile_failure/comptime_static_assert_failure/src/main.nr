use std::static_assert;

comptime fn foo(x: Field) -> bool {
    static_assert(x == 4, "x != 4");
    x == 4
}

fn main() {
    comptime {
        static_assert(foo(3), "expected message");
    }
}

