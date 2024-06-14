// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
// Returns the merkle root of the tree from the provided leaf, its hashpath, using a pedersen hash function.
pub fn compute_merkle_root<N>(leaf: Field, index: Field, hash_path: [Field; N]) -> Field {
    let n = hash_path.len();
    let index_bits = index.to_le_bits(n as u32);
    let mut current = leaf;
    for i in 0..n {
        let path_bit = index_bits[i] as bool;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
        current = dep::std::hash::pedersen_hash([hash_left, hash_right]);
    }
    current
}


mod tests {
    use crate::compute_merkle_root;

    #[test]
    fn test_insert() {
        let old_root: Field = 0x2131c74af6250d1246797588ff075b672e3d2e6805f9c6b313ade8f6b9e12802;
        let old_leaf: Field = 0x2df8b940e5890e4e1377e05373fae69a1d754f6935e6a780b666947431f2cdcd;
        let old_hash_path: [Field; 3] = [
            0x03542cb720369f19a74fd05b4edfbedb27a78514ad3283f1b3270a1656cced8e,
            0x1278272f6e617666b18c52349c4bbe94249a8ab59add2ee0a142168a92b7ffc2,
            0x2547cfd7699ad1f331fb77b30a3445c6043a4c62b5b1215356b2924607bf937b,
        ];
        let new_root: Field = 0x25e2a8ee5b85e5b546ae27832b9b53f5fae5b371e3e7f8cddda839f41669fc68;
        let leaf: Field = 0x23fe6c8f2426b793f0f156f57efbecbea52e951af761634a85e80cc1a9fb5003;
        let index: Field = 0;

        assert(old_root == compute_merkle_root(old_leaf, index, old_hash_path));

        let calculated_root = compute_merkle_root(leaf, index, old_hash_path);
        assert(new_root == calculated_root);
    }
}