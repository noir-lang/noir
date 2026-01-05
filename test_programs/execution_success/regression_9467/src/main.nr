fn main(b: bool) -> pub (u32, u32) {
    let s0: [u32] = #[];
    let s1: [u32] = #[10];
    let s2 = if b { s1 } else { s0 };
    let i1 = if b { s0.pop_back().1 } else { 1 };
    let i2 = if b { s2.pop_front().0 } else { 2 };
    (i1, i2)
}
