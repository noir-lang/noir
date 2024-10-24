fn main() {
    // s: Struct<?PolymorphicIntOrField, ()>
    let s = Struct { a: 0, b: () };
    // Regression for #3089
    s.foo();
}

struct Struct<A, B> {
    a: A,
    b: B,
}

// Before the fix, this candidate is searched first, binding ? to `u8` permanently.
impl Struct<u8, u8> {
    fn foo(self) {
        let _ = self;
    }
}

// Then this candidate would be searched next but would not be a valid
// candidate since `Struct<u32, ()>` != `Struct<u8, ()>`.
//
// With the fix, the type of `s` correctly no longer changes until a
// method is actually selected. So this candidate is now valid since
// `Struct<u32, ()>` unifies with `Struct<?, ()>` with `? = u32`.
impl Struct<u32, ()> {
    fn foo(self) {
        let _ = self;
    }
}
