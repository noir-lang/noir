use std::mem::checked_transmute;

fn main() {
    // 1*(2 + 3) = 1*2 + 1*3 = 5
    let _: [Field; 5] = distribute::<1, 2, 3>([1, 2, 3, 4, 5]);
}

pub fn distribute<let N: u32, let A: u32, let B: u32>(
    x: [Field; N * (A + B)],
) -> [Field; N * A + N * B] {
    // asserts: [Field; N * (A + B)] = [Field; N * A + N * B]
    //       -> N * A + B = N * A + N * B
    //
    // This assert occurs during monomorphization when the actual values for N, A, and B
    // become known. This also means if this function is not called, the assert will not trigger.
    checked_transmute(x)
}
