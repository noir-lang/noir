
        fn main(i: u64) -> pub u64 {
            if i == 0 { 0 } else { i + main(i-1) }
        }
        