// This test prevents users from trying to create their own blackbox functions as these should only exist in the stdlib.

// This would otherwise be a perfectly valid definition of the `pedersen_hash` black box function,
// however executing the circuit results in an unhelpful ICE.
#[foreign(pedersen_hash)]
fn my_pedersen_hash<let N: u32>(_input: [Field; N]) -> Field {}

fn main() -> pub Field {
    my_pedersen_hash([1])
}
