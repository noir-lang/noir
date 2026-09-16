// The ownership pass must keep the clone protecting a by-value array argument
// when a *later* argument of the same call mutates that array through a
// `&mut`. `sha256_compression` must hash `x` as it was before
// `bump` ran; without the clone, Brillig's copy-on-write writes in place and the
// intrinsic hashes the mutated buffer, diverging from ACIR.
fn bump(x: &mut [u32; 16], i: u32) -> [u32; 8] {
    (*x)[i] = 7;
    [1, 2, 3, 4, 5, 6, 7, 8]
}

fn core(i: u32) -> u64 {
    let mut x = [i + 1; 16];
    let out = std::hash::sha256_compression(x, bump(&mut x, i));
    out[0] as u64 + x[i] as u64
}

fn main(i: u32) -> pub u64 {
    core(i)
}
