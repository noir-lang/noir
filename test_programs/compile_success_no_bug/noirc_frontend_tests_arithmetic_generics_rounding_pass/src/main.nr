fn main() {
    // 3/2*2 = 2
    round::<3, 2>([1, 2]);
}

fn round<let N: u32, let M: u32>(_x: [Field; N / M * M]) {}
