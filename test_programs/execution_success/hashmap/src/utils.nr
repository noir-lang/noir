// Compile-time: cuts the M first elements from the BoundedVec<T, N>.
pub(crate) fn cut<T, N, M>(input: BoundedVec<T, N>) -> [T; M] {
    assert(M < N, "M should be less than N.");

    let mut new = BoundedVec::new();
    for i in 0..M {
        new.push(input.get(i));
    }
    new.storage()
}
