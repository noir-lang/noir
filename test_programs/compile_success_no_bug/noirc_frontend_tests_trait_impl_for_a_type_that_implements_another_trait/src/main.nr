
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            self
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    pub fn use_it<T>(t: T) -> i32 where T: Two {
        Two::two(t)
    }

    fn main() {}
    