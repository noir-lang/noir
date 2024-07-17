use crate::{traits::Empty, hash::accumulate_sha256};
// N = maximum leaves
// For now we only care about the root
struct VariableMerkleTree {
    // leaves: [Field; N],
    root: Field,
}

unconstrained fn get_height(input: u32, start: u8) -> u8 {
    let mut height = 0;
    if input <= 2 << start {
        height = start;
    } else {
        height = get_height(input, start + 1);
    }
    height
}

// This calculates the root of the minimal size merkle tree required
// to store num_non_empty_leaves
// Since we cannot isolate branches, it doesn't cost fewer gates than using
// MerkleTree on the full array of elements N, but is slightly cheaper on-chain
// and cleaner elsewhere.
impl<let N: u32> VariableMerkleTree {
    // Example - tx_0 with 3 msgs | tx_1 with 2 msgs creates:
    //               
    //        tx0oh           tx1oh
    //     /          \      /    \    
    //   msg01     msg23    msg3 msg4
    //  /     \    /    \  
    // msg0 msg1 msg2    0  
    // |       tx_0      |  | tx_1 |
    //
    pub fn new_sha(leaves: [Field; N], num_non_empty_leaves: u32) -> Self {
        // Find size of tree required
        let height = get_height(num_non_empty_leaves, 0);
        let next_power_2 = 2 << height;
        let prev_power_2 = next_power_2 / 2;
        // If we have no num_non_empty_leaves, we use a 'tree' of 2 leaves
        // TODO perhaps we just return 0 in this case?
        assert(
            (num_non_empty_leaves == 0)
            | (num_non_empty_leaves == 1)
            | (num_non_empty_leaves > prev_power_2)
        );
        assert(num_non_empty_leaves <= next_power_2);
        // hash base layer
        let mut stop = false;
        let mut nodes = [0; N];
        for i in 0..N / 2 {
            // stop after non zero leaves
            if i == prev_power_2 {
                stop = true;
            }
            if (!stop) {
                nodes[i] = accumulate_sha256([leaves[2*i], leaves[2*i+1]]);
            }
        }

        // hash the other layers
        stop = if prev_power_2 == 1 {true} else {false};
        let mut next_layer_end = prev_power_2 / 2;
        let mut next_layer_size = next_layer_end;
        let mut root = nodes[0];
        for i in 0..(N - 1 - N / 2) {
            if !stop {
                nodes[prev_power_2+i] = accumulate_sha256([nodes[2*i], nodes[2*i+1]]);
                if i == next_layer_end {
                    // Reached next layer => move up one layer
                    next_layer_size = next_layer_size/2;
                    next_layer_end += next_layer_size;
                }
                if (next_layer_size == 1) {
                    // Reached root
                    root = nodes[prev_power_2+i];
                    stop = true;
                }
            }
        }

        VariableMerkleTree { root }
    }

    fn get_root(self) -> Field {
        self.root
    }
}

mod tests {
    use crate::{
        merkle_tree::{variable_merkle_tree::VariableMerkleTree, merkle_tree::MerkleTree},
        hash::accumulate_sha256
    };

    fn generate_test_array(non_empty_items: u32) -> [Field; 100] {
        let mut items: [Field; 100] = [0; 100];
        let mut stop = false;
        for i in 0..100 {
            if i == non_empty_items {
                stop = true;
            }
            if !stop {
                items[i] = 10 + i as Field;
            }
        }
        items
    }

    pub fn generate_full_sha_tree<let N: u32>(leaves: [Field; N]) -> MerkleTree<N> {
        let mut nodes = [0; N];
        let total_nodes = N - 1;
        let half_size = N / 2;
        // hash base layer
        for i in 0..half_size {
            nodes[i] = accumulate_sha256([leaves[2*i], leaves[2*i+1]]);
        }
        // hash the other layers
        for i in 0..(total_nodes - half_size) {
            nodes[half_size+i] = accumulate_sha256([nodes[2*i], nodes[2*i+1]]);
        }
        MerkleTree { leaves, nodes }
    }

    #[test]
    fn test_0_elems() {
        let items = [0; 100];
        let expected_root = accumulate_sha256([items[0], items[1]]);
        let tree = VariableMerkleTree::new_sha(items, 0);
        assert_eq(tree.root, expected_root);
    }

    #[test]
    fn test_1_elems() {
        let items = generate_test_array(1);
        let expected_root = accumulate_sha256([items[0], items[1]]);
        let tree = VariableMerkleTree::new_sha(items, 1);
        assert_eq(tree.root, expected_root);
    }

    #[test]
    fn test_2_elems() {
        let items = generate_test_array(2);
        let expected_root = accumulate_sha256([items[0], items[1]]);
        let tree = VariableMerkleTree::new_sha(items, 2);
        assert_eq(tree.root, expected_root);
    }

    #[test]
    fn test_3_elems() {
        let items = generate_test_array(3);
        let expected_node_0 = accumulate_sha256([items[0], items[1]]);
        let expected_node_1 = accumulate_sha256([items[2], 0]);
        let expected_root = accumulate_sha256([expected_node_0, expected_node_1]);
        let tree = VariableMerkleTree::new_sha(items, 3);
        assert_eq(tree.root, expected_root);
    }

    #[test]
    fn test_5_elems() {
        let items = generate_test_array(5);
        let mut items_pwr_2 = [0; 8];
        for i in 0..8 {
            items_pwr_2[i] = items[i];
        }
        let full_tree = generate_full_sha_tree(items_pwr_2);
        let tree = VariableMerkleTree::new_sha(items, 5);
        assert_eq(tree.root, full_tree.get_root());
    }

    #[test]
    fn test_10_elems() {
        let items = generate_test_array(10);
        let mut items_pwr_2 = [0; 16];
        for i in 0..16 {
            items_pwr_2[i] = items[i];
        }
        let full_tree = generate_full_sha_tree(items_pwr_2);
        let tree = VariableMerkleTree::new_sha(items, 10);
        assert_eq(tree.root, full_tree.get_root());
    }

    #[test]
    fn test_30_elems() {
        let items = generate_test_array(30);
        let mut items_pwr_2 = [0; 32];
        for i in 0..32 {
            items_pwr_2[i] = items[i];
        }
        let full_tree = generate_full_sha_tree(items_pwr_2);
        let tree = VariableMerkleTree::new_sha(items, 30);
        assert_eq(tree.root, full_tree.get_root());
    }

    #[test]
    fn test_61_elems() {
        let items = generate_test_array(61);
        let mut items_pwr_2 = [0; 64];
        for i in 0..64 {
            items_pwr_2[i] = items[i];
        }
        let full_tree = generate_full_sha_tree(items_pwr_2);
        let tree = VariableMerkleTree::new_sha(items, 61);
        assert_eq(tree.root, full_tree.get_root());
    }
}

