
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            let _ = self;
            1
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

    impl Two for u32 {
        fn two(self) -> i32 {
            let _ = self;
            0
        }
    }

    pub fn use_it(t: u32) -> i32 {
        Two::two(t)
    }

    fn main() {}
    