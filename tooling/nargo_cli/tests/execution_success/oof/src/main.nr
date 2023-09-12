unconstrained fn main(
    data: Field,
) -> pub [u1;2048]{
    let mut bits = [0; 2048];
    for i in 0..2048 {
        if i != data {
            bits[i] = 1;
        }
    }
    bits
}
