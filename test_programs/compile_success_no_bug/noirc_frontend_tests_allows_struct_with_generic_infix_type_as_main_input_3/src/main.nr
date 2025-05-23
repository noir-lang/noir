struct Foo<let N: u32> {
    x: [u64; N * 2],
}

global N: u32 = 9;

fn main(_x: Foo<N * 2>) {}
