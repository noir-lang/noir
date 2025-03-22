
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    trait Test {
        fn test(self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Test for u64 {
        fn test(self) -> bool { self == self }
    }

    fn test_eq(x: impl Eq, y: impl Test) -> bool {
        x.eq(x) == y.test()
    }

    fn main(a: Foo, b: u64) -> pub bool {
        test_eq(a, b)
    }