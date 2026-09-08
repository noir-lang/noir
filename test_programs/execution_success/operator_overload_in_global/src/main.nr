pub struct U<let N: u32> {
    limbs: [Field; N],
}

impl<let N: u32> std::ops::Add for U<N> {
    fn add(self, other: Self) -> Self {
        let mut r: [Field; N] = [0; N];
        for i in 0..N {
            r[i] = self.limbs[i] + other.limbs[i];
        }
        U { limbs: r }
    }
}

fn build<let N: u32>(a: [Field; N]) -> [Field; N] {
    let u: U<N> = U { limbs: a };
    (u + u).limbs
}

global G: [Field; 2] = build([1, 2]);

fn main(x: Field) {
    assert(G[0] == 2);
    assert(G[1] == 4);
    assert(x == 1);
}
