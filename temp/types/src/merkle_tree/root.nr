use crate::{hash::merkle_hash, merkle_tree::merkle_tree::MerkleTree};

// Calculate the Merkle tree root from the sibling path and leaf.
//
// The leaf is hashed with its sibling, and then the result is hashed
// with the next sibling etc in the path. The last hash is the root.
//
// TODO(David/Someone): The cpp code is using a uint256, whereas its
// TODO a bit simpler in Noir to just have a bit array.
// TODO: I'd generally like to avoid u256 for algorithms like
// this because it means we never even need to consider cases where
// the index is greater than p.
pub fn root_from_sibling_path<let N: u32>(
    leaf: Field,
    leaf_index: Field,
    sibling_path: [Field; N],
) -> Field {
    let mut node = leaf;
    let indices: [u1; N] = leaf_index.to_le_bits();

    for i in 0..N {
        let (hash_left, hash_right) = if indices[i] == 1 {
            (sibling_path[i], node)
        } else {
            (node, sibling_path[i])
        };
        node = merkle_hash(hash_left, hash_right);
    }
    node
}

pub fn calculate_subtree_root<let N: u32>(leaves: [Field; N]) -> Field {
    MerkleTree::new(leaves).get_root()
}

// These values are precomputed and we run tests to ensure that they
// are correct. The values themselves were computed from the cpp code.
//
// Would be good if we could use width since the compute_subtree
// algorithm uses depth.
pub fn calculate_empty_tree_root(depth: u32) -> Field {
    if depth == 0 {
        0
    } else if depth == 1 {
        0x0b63a53787021a4a962a452c2921b3663aff1ffd8d5510540f8e659e782956f1
    } else if depth == 2 {
        0x0e34ac2c09f45a503d2908bcb12f1cbae5fa4065759c88d501c097506a8b2290
    } else if depth == 3 {
        0x21f9172d72fdcdafc312eee05cf5092980dda821da5b760a9fb8dbdf607c8a20
    } else if depth == 4 {
        0x2373ea368857ec7af97e7b470d705848e2bf93ed7bef142a490f2119bcf82d8e
    } else if depth == 5 {
        0x120157cfaaa49ce3da30f8b47879114977c24b266d58b0ac18b325d878aafddf
    } else if depth == 6 {
        0x01c28fe1059ae0237b72334700697bdf465e03df03986fe05200cadeda66bd76
    } else if depth == 7 {
        0x2d78ed82f93b61ba718b17c2dfe5b52375b4d37cbbed6f1fc98b47614b0cf21b
    } else if depth == 8 {
        0x067243231eddf4222f3911defbba7705aff06ed45960b27f6f91319196ef97e1
    } else if depth == 9 {
        0x1849b85f3c693693e732dfc4577217acc18295193bede09ce8b97ad910310972
    } else if depth == 10 {
        0x2a775ea761d20435b31fa2c33ff07663e24542ffb9e7b293dfce3042eb104686
    } else {
        panic(f"depth should be between 0 and 10")
    }
}

#[test]
fn test_merkle_root_interop_test() {
    // This is a test to ensure that we match the cpp implementation.
    // You can grep for `TEST_F(root_rollup_tests, noir_interop_test)`
    // to find the test that matches this.
    let root = calculate_subtree_root([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]);
    assert(0x1a09d935ae110b4c861fcec8f9099ec30b4485022aeb3d3cf9d7168e38fdc231 == root);

    let empty_root = calculate_subtree_root([0; 16]);
    assert(0x2373ea368857ec7af97e7b470d705848e2bf93ed7bef142a490f2119bcf82d8e == empty_root);
}

#[test]
fn test_empty_subroot() {
    assert(calculate_empty_tree_root(0) == 0);

    let expected_empty_root_2 = calculate_subtree_root([0; 2]);
    assert(calculate_empty_tree_root(1) == expected_empty_root_2);

    let expected_empty_root_4 = calculate_subtree_root([0; 4]);
    assert(calculate_empty_tree_root(2) == expected_empty_root_4);

    let expected_empty_root_8 = calculate_subtree_root([0; 8]);
    assert(calculate_empty_tree_root(3) == expected_empty_root_8);

    let expected_empty_root_16 = calculate_subtree_root([0; 16]);
    assert(calculate_empty_tree_root(4) == expected_empty_root_16);

    let expected_empty_root_32 = calculate_subtree_root([0; 32]);
    assert(calculate_empty_tree_root(5) == expected_empty_root_32);

    let expected_empty_root_64 = calculate_subtree_root([0; 64]);
    assert(calculate_empty_tree_root(6) == expected_empty_root_64);

    let expected_empty_root_128 = calculate_subtree_root([0; 128]);
    assert(calculate_empty_tree_root(7) == expected_empty_root_128);

    let expected_empty_root_256 = calculate_subtree_root([0; 256]);
    assert(calculate_empty_tree_root(8) == expected_empty_root_256);

    let expected_empty_root_512 = calculate_subtree_root([0; 512]);
    assert(calculate_empty_tree_root(9) == expected_empty_root_512);

    let expected_empty_root_1024 = calculate_subtree_root([0; 1024]);
    assert(calculate_empty_tree_root(10) == expected_empty_root_1024);
}
