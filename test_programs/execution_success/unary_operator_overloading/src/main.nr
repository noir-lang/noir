use std::ops::{Neg, Not};

// x = 3
fn main(x: u32) {
    let wx = Wrapper::new(x as i32);
    let ex: i32 = 3;

    assert((-wx).inner == -ex);
    assert((!wx).inner == !ex);

    // Check that it works with type variables (x's type isn't immediately known)
    let x = 3;
    assert(-3 == -x);
}

struct Wrapper {
    inner: i32,
}

impl Wrapper {
    fn new(inner: i32) -> Self {
        Wrapper { inner }
    }
}

impl Neg for Wrapper {
    fn neg(self) -> Wrapper {
        Wrapper::new(-self.inner)
    }
}

impl Not for Wrapper {
    fn not(self) -> Wrapper {
        Wrapper::new(!self.inner)
    }
}
