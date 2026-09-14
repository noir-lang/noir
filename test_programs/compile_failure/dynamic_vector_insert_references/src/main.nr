// Inserting into a vector of references at a dynamic index is a dynamic selection of a
// reference, which cannot be resolved before ACIR generation. This must be rejected with a
// clean error rather than reaching a later pass that panics on the leftover memory operations.
fn main(i: u32) -> pub Field {
    let mut x = 10;
    let mut y = 20;

    let v: [&mut Field] = [&mut x, &mut y].as_vector();
    let v2 = v.insert(i, &mut x);

    y = 77;
    *v2[0]
}
