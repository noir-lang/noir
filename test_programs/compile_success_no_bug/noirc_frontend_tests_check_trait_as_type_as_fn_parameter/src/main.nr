trait Eq2 {
    fn eq2(self, other: Self) -> bool;
}

struct Foo {
    a: u64,
}

impl Eq2 for Foo {
    fn eq2(self, other: Foo) -> bool {
        self.a == other.a
    }
}

fn test_eq(x: impl Eq2) -> bool {
    x.eq2(x)
}

fn main(a: Foo) -> pub bool {
    test_eq(a)
}
