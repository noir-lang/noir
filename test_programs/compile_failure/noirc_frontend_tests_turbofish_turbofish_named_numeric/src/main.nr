
    trait Bar {
        let N: u32;
    }

    impl Bar for Field {
        let N: u32 = 1;
    }

    impl Bar for i32 {
        let N: u32 = 2;
    }

    fn foo<B>()
    where
        B: Bar<N = 1>,
    {}

    fn main() {
        foo::<Field>();
        foo::<i32>();
    }
    