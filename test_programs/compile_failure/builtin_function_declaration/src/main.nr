// This test prevents users from trying to create their own builtin functions as these should only exist in the stdlib.

// This would otherwise be a perfectly valid declaration of the `to_le_bits` builtin function
#[builtin(to_le_bits)]
fn to_le_bits(_x: Field, _bit_size: u32) -> [u1] {}

fn main(x: Field) -> pub u1 {
    let bits = to_le_bits(x, 100);
    bits[0]
}
