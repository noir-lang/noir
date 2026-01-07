struct MyStruct {
    x: &mut Field,
    y: u32,
}

fn main(safe: bool) -> pub u32 {
    let mut x = 1;
    let s = MyStruct { x: &mut x, y: 2 };
    foo(safe, || s.y)
}

fn foo<Env, T>(safe: bool, f: fn[Env]() -> T) -> T {
    if safe {
        f()
    } else {
        // Safety: this is actually unsafe, potentially trying to pass an &mut from ACIR to Brillig.
        unsafe {
            bar(f)
        }
    }
}

unconstrained fn bar<Env, T>(f: fn[Env]() -> T) -> T {
    f()
}
