fn main() {
    comptime {
        let y = foo::<340282366920938463463374607431768211455, 1>();
        assert_eq(y.n(), 340282366920938463463374607431768211454);
    }
}

struct T<let N: u128> {}

impl<let N: u128> T<N> {
    fn n(self) -> u128 {
        let _ = self;
        N
    }
}

fn foo<let x: u128, let y: u128>() -> T<x - y> {
    T {}
}
