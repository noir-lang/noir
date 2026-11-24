// Passes the same large array to an unconstrained helper once per block, the pattern of a
// block-based hash such as sha256. The ACIR bytecode size of each helper call depends on how
// the array is passed, so this program tracks that cost in the artifact size.
global N: u32 = 4096;
global BLOCK: u32 = 64;

unconstrained fn block_sum(msg: [u8; N], start: u32) -> u32 {
    let mut sum = 0;
    for k in 0..BLOCK {
        sum += msg[start + k] as u32;
    }
    sum
}

fn main(msg: [u8; N]) -> pub u32 {
    let mut total = 0;
    for i in 0..N / BLOCK {
        // Safety: the result is checked against the same sum computed in the circuit.
        let sum = unsafe { block_sum(msg, i * BLOCK) };
        let mut expected = 0;
        for k in 0..BLOCK {
            expected += msg[i * BLOCK + k] as u32;
        }
        assert_eq(sum, expected);
        total += sum;
    }
    total
}
