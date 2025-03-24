struct Foo<let N: u32> {
    x: [u64; N * 2],
}

fn main(_x: Foo<2 * 9>) {}
