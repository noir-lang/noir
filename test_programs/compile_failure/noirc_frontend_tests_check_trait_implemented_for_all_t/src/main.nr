
    trait Default {
        fn default() -> Self;
    }

    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    trait IsDefault {
        fn is_default(self) -> bool;
    }

    impl<T> IsDefault for T where T: Default + Eq {
        fn is_default(self) -> bool {
            self.eq(T::default())
        }
    }

    struct Foo {
        a: u64,
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Default for u64 {
        fn default() -> Self {
            0
        }
    }

    impl Default for Foo {
        fn default() -> Self {
            Foo { a: Default::default() }
        }
    }

    fn main(a: Foo) -> pub bool {
        a.is_default()
    }