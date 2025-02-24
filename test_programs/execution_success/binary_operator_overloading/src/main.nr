use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

// x = 3, y = 9
fn main(x: u32, y: u32) {
    let wx = Wrapper::new(x);
    let wy = Wrapper::new(y);

    // expected x and expected y values
    let ex: u32 = 3;
    let ey: u32 = 9;

    assert((wx + wy).inner == ex + ey);
    assert((wy - wx).inner == ey - ex);
    assert((wx * wy).inner == ex * ey);
    assert((wx / wy).inner == ex / ey);
    assert((wx % wy).inner == ex % ey);

    assert((wx & wy).inner == (ex & ey));
    assert((wx | wy).inner == (ex | ey));
    assert((wx ^ wy).inner == (ex ^ ey));

    assert((wy << x as u8).inner == (ey << ex as u8));
    assert((wy >> x as u8).inner == (ey >> ex as u8));

    assert((wx == wy) == (ex == ey));
    assert((wx < wy) == (ex < ey));
    assert((wx <= wy) == (ex <= ey));
    assert((wx > wy) == (ex > ey));
    assert((wx >= wy) == (ex >= ey));
    assert(wx.cmp(wy) == ex.cmp(ey));

    // Ensure operator overloading still works with more complex types
    let pair_ascending = Pair { x: wx, y: wy };
    let pair_descending = Pair { x: wy, y: wx };

    assert(pair_ascending != pair_descending);

    assert(pair_ascending < pair_descending);
    assert(pair_ascending <= pair_descending);
    assert(pair_descending > pair_ascending);
    assert(pair_descending >= pair_ascending);

    assert(pair_ascending.cmp(pair_descending) == Ordering::less());
}

struct Wrapper {
    inner: u32,
}

impl Wrapper {
    fn new(inner: u32) -> Self {
        Wrapper { inner }
    }
}

impl Add for Wrapper {
    fn add(self, other: Self) -> Self {
        Wrapper::new(self.inner + other.inner)
    }
}

impl Sub for Wrapper {
    fn sub(self, other: Self) -> Self {
        Wrapper::new(self.inner - other.inner)
    }
}

impl Mul for Wrapper {
    fn mul(self, other: Self) -> Self {
        Wrapper::new(self.inner * other.inner)
    }
}

impl Div for Wrapper {
    fn div(self, other: Self) -> Self {
        Wrapper::new(self.inner / other.inner)
    }
}

impl Rem for Wrapper {
    fn rem(self, other: Self) -> Self {
        Wrapper::new(self.inner % other.inner)
    }
}

impl BitAnd for Wrapper {
    fn bitand(self, other: Self) -> Self {
        Wrapper::new(self.inner & other.inner)
    }
}

impl BitOr for Wrapper {
    fn bitor(self, other: Self) -> Self {
        Wrapper::new(self.inner | other.inner)
    }
}

impl BitXor for Wrapper {
    fn bitxor(self, other: Self) -> Self {
        Wrapper::new(self.inner ^ other.inner)
    }
}

impl Shl for Wrapper {
    fn shl(self, other: u8) -> Self {
        Wrapper::new(self.inner << other)
    }
}

impl Shr for Wrapper {
    fn shr(self, other: u8) -> Self {
        Wrapper::new(self.inner >> other)
    }
}

impl Eq for Wrapper {
    fn eq(self, other: Self) -> bool {
        self.inner == other.inner
    }
}

impl Ord for Wrapper {
    fn cmp(self, other: Self) -> Ordering {
        self.inner.cmp(other.inner)
    }
}

struct Pair {
    x: Wrapper,
    y: Wrapper,
}

impl Eq for Pair {
    fn eq(self, o: Self) -> bool {
        (self.x == o.x) & (self.y == o.y)
    }
}

impl Ord for Pair {
    fn cmp(self, o: Self) -> Ordering {
        let mut result = self.x.cmp(o.x);

        if result == Ordering::equal() {
            result = self.y.cmp(o.y);
        }

        result
    }
}
