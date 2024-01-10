
fn main() {
    let xs: [Field; 1] = [3];
    let ys: [u32; 1] = [3];
    foo(xs, ys);
}

// Bug: `U: Eq` constraint is not required
// Bug: `x.into() == u` causes a type mismatch `_ != U`
fn foo<T, U>(x: T, u: U) where T: Into<U> {
    assert(Eq::eq(x.into(), u));
}

trait Into<T> {
    fn into(self) -> T;
}

impl<T, U, N> Into<[U; N]> for [T; N] where T: Into<U> {
    fn into(self) -> [U; N] {
        self.map(|x: T| x.into())
    }
}

impl Into<u32> for Field {
    fn into(self) -> u32 {
        self as u32
    }
}
