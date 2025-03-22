
        fn main() -> pub u64 {
            foo(1, main())
        }
        fn foo(a: u64, b: u64) -> u64 {
            a + b
        }
        