
        type Outer<let N: u32> = [u8; N];
        fn main(_arg: Outer<1>) {}
    