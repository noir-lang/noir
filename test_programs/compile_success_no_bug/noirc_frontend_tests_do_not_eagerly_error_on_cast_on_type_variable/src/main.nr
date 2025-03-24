pub fn foo<T, U>(x: T, f: fn(T) -> U) -> U {
    f(x)
}

fn main() {
    let x: u8 = 1;
    let _: Field = foo(x, |x| x as Field);
}
