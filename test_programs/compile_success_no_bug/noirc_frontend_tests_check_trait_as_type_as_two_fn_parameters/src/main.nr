trait Eq2 {
    fn eq2(self, other: Self) -> bool;
}

trait Test {
    fn test(self) -> bool;
}

struct Foo {
    a: u64,
}

impl Eq2 for Foo {
    fn eq2(self, other: Foo) -> bool {
        self.a == other.a
    }
}

impl Test for u64 {
    fn test(self) -> bool {
        self == self
    }
}

fn test_eq(x: impl Eq2, y: impl Test) -> bool {
    x.eq2(x) == y.test()
}

fn main(a: Foo, b: u64) -> pub bool {
    test_eq(a, b)
}
