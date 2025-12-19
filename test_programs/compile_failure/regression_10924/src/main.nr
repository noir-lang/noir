// This test should fail, but it should not panic the compiler.
fn main() -> pub u32 {
    comptime {
        let a = 10;
        foo();
        a
    }
}

fn foo() -> pub u32 {
    comptime {
        let f = |a: u32, b: u32| a + b;
        let _ = f(1);
    }
}
