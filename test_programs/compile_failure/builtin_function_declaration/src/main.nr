// This test prevents users from trying to create their own builtin functions as these should only exist in the stdlib.

// This would otherwise be a perfectly valid declaration of the `to_le_bits` builtin function
#[builtin(to_le_bits)]
fn to_le_bits<let N: u32>(_x: Field) -> [bool; N] {}

fn main(x: Field) -> pub bool {
    let bits: [bool; 100] = to_le_bits(x);
    bits[0]
}
