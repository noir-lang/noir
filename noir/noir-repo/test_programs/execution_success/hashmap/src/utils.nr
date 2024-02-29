// Compile-time: cuts the M first elements from the [T; N] array.
pub(crate) fn cut<T, N, M>(input: [T; N]) -> [T; M] {
    assert(M as u64 < N as u64, "M should be less than N.");

    let mut new = [dep::std::unsafe::zeroed(); M];
    for i in 0..M {
        new[i] = input[i];
    }
    new
}
