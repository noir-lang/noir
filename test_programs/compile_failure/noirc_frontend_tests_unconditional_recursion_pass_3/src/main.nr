
        fn main() -> pub u64 {
            let mut sum = 0;
            for _ in 0 .. 10 {
                sum += main();
            }
            sum
        }
        