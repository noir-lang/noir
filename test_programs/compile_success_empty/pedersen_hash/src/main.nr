use dep::std;

fn main() {
    let hash = std::hash::pedersen_hash([0, 1]);
    assert_eq(hash, 0x0d98561fb02ca04d00801dfdc118b2a24cea0351963587712a28d368041370e1);
}

