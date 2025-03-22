
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a }
    }

    fn test_eq(x: impl Eq) -> bool {
        x.eq(x)
    }

    fn main(a: Foo) -> pub bool {
        test_eq(a)
    }