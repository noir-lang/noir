use std::static_assert;

comptime fn foo(x: Field) -> bool {
    static_assert(x == 4, "x != 4");
    x == 4
}

global C: bool = {
    let out = foo(2 + 2);
    static_assert(out, "foo did not pass in C");
    out
};

fn main() {
    comptime {
        static_assert(foo(4), "foo did not pass in main");
        static_assert(C, "C did not pass")
    }
}
