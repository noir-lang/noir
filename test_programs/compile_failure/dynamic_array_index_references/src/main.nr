// Reading an array of references at a dynamic index is a dynamic selection of a reference, which
// cannot be resolved before ACIR generation. Rejecting it here is what keeps the address out of
// ACIR: were it to survive, mem2reg would have no way to promote the allocation behind it and a
// later pass would hit its "no memory operations in ACIR" assertion instead of reporting anything
// a user can act on.
fn main(i: u32) -> pub Field {
    let mut x = 10;
    let mut y = 20;

    let r: [&mut Field; 2] = [&mut x, &mut y];

    y = 77;
    *r[i]
}
