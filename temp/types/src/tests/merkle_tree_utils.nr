use crate::{
    hash::merkle_hash,
    merkle_tree::{calculate_empty_tree_root, merkle_tree, MerkleTree},
    traits::Empty,
};

pub fn compute_zero_hashes<let N: u32>(mut hashes: [Field; N]) -> [Field; N] {
    hashes[0] = merkle_hash(0, 0);

    for i in 1..N {
        hashes[i] = merkle_hash(hashes[i - 1], hashes[i - 1]);
    }

    hashes
}

impl<let N: u32> MerkleTree<N> {
    pub fn update_leaf<let K: u32>(&mut self, index: u32, value: Field, _tree_height: [Field; K]) {
        self.leaves[index] = value;

        let mut sibling_index = merkle_tree::sibling_index(index);
        let (mut left_node, mut right_node) = if index % 2 == 0 {
            (value, self.leaves[sibling_index])
        } else {
            (self.leaves[sibling_index], value)
        };

        let mut current_width: u32 = N / 2;
        let mut layer_offset: u32 = 0;
        let mut node_index: u32 = index / 2 + layer_offset;
        for _ in 0..K {
            self.nodes[node_index] = merkle_hash(left_node, right_node);
            sibling_index = merkle_tree::sibling_index(node_index);
            let nodes = if node_index % 2 == 0 {
                (self.nodes[node_index], self.nodes[sibling_index])
            } else {
                (self.nodes[sibling_index], self.nodes[node_index])
            };
            left_node = nodes.0;
            right_node = nodes.1;

            let current_index_at_layer = node_index - layer_offset;

            layer_offset += current_width;
            node_index = (current_index_at_layer / 2) + layer_offset;
            current_width = current_width / 2;
        }
    }
}

#[test]
fn test_merkle_tree_update_leaf() {
    let mut tree = MerkleTree::new([1, 2]);
    assert_eq(tree.get_root(), merkle_hash(1, 2));
    tree.update_leaf(1, 0, [0; 1]);
    assert_eq(tree.get_root(), merkle_hash(1, 0));
    tree.update_leaf(0, 0, [0; 1]);
    assert_eq(tree.get_root(), merkle_hash(0, 0));
}

#[test]
fn test_merkle_tree_update_leaf_three_layers() {
    let mut tree = MerkleTree::new([1, 2, 3, 4, 5, 6, 7, 8]);

    for i in 0..8 {
        tree.update_leaf(i, 0, [0; 3]);
    }

    assert_eq(tree.get_root(), calculate_empty_tree_root(3));
}

pub struct NonEmptyMerkleTree<let SUBTREE_ITEMS: u32, let TREE_HEIGHT: u32, let SUPERTREE_HEIGHT: u32, let SUBTREE_HEIGHT: u32> {
    subtree: MerkleTree<SUBTREE_ITEMS>,
    zero_hashes: [Field; TREE_HEIGHT],
    left_supertree_branch: [Field; SUPERTREE_HEIGHT],
    _phantom_subtree_height: [Field; SUBTREE_HEIGHT],
}

impl<let SUBTREE_ITEMS: u32, let TREE_HEIGHT: u32, let SUPERTREE_HEIGHT: u32, let SUBTREE_HEIGHT: u32> Empty for NonEmptyMerkleTree<SUBTREE_ITEMS, TREE_HEIGHT, SUPERTREE_HEIGHT, SUBTREE_HEIGHT> {
    fn empty() -> Self {
        NonEmptyMerkleTree {
            subtree: MerkleTree::empty(),
            zero_hashes: [0; TREE_HEIGHT],
            left_supertree_branch: [0; SUPERTREE_HEIGHT],
            _phantom_subtree_height: [0; SUBTREE_HEIGHT],
        }
    }
}

impl<let SUBTREE_ITEMS: u32, let TREE_HEIGHT: u32, let SUPERTREE_HEIGHT: u32, let SUBTREE_HEIGHT: u32> NonEmptyMerkleTree<SUBTREE_ITEMS, TREE_HEIGHT, SUPERTREE_HEIGHT, SUBTREE_HEIGHT> {
    pub fn new(
        non_zero_leaves: [Field; SUBTREE_ITEMS],
        _tree_height: [Field; TREE_HEIGHT],
        _supertree_height: [Field; SUPERTREE_HEIGHT],
        _subtree_height: [Field; SUBTREE_HEIGHT],
    ) -> Self {
        // Hack to get around us converting a u32 to a u8.
        // TODO: improve this.
        (SUBTREE_HEIGHT as Field).assert_max_bit_size::<8>();

        assert_eq(
            TREE_HEIGHT,
            SUPERTREE_HEIGHT + SUBTREE_HEIGHT,
            "tree height must be the sum of supertree and subtree height",
        );
        assert_eq(
            SUBTREE_ITEMS as u128,
            (1 << SUBTREE_HEIGHT as u8) as u128,
            "subtree items must be 2^subtree height",
        );
        let subtree = MerkleTree::new(non_zero_leaves);

        let zero_hashes = compute_zero_hashes(_tree_height);

        let mut left_supertree_branch = [0; SUPERTREE_HEIGHT];
        left_supertree_branch[0] = merkle_hash(subtree.get_root(), zero_hashes[SUBTREE_HEIGHT - 1]);
        for i in 1..left_supertree_branch.len() {
            // TODO(md): far right of this yuck
            left_supertree_branch[i] = merkle_hash(
                left_supertree_branch[i - 1],
                zero_hashes[SUBTREE_HEIGHT - 1 + i],
            );
        }

        NonEmptyMerkleTree {
            subtree,
            zero_hashes,
            left_supertree_branch,
            _phantom_subtree_height: _subtree_height,
        }
    }

    pub fn get_sibling_path(self, leaf_index: u32) -> [Field; TREE_HEIGHT] {
        let mut path = [0; TREE_HEIGHT];
        let mut current_index = leaf_index;
        let mut subtree_width = SUBTREE_ITEMS;

        let mut sibling_index = merkle_tree::sibling_index(current_index);

        path[0] = if current_index < subtree_width {
            self.subtree.leaves[sibling_index]
        } else {
            0
        };

        let mut subtree_offset: u32 = 0;

        for i in 1..TREE_HEIGHT {
            current_index = current_index / 2;
            subtree_width = subtree_width / 2;

            sibling_index = merkle_tree::sibling_index(current_index);

            if sibling_index < subtree_width {
                path[i] = self.subtree.nodes[subtree_offset + sibling_index];
            } else if sibling_index == 0 {
                path[i] = self.left_supertree_branch[i - 1 - SUBTREE_HEIGHT];
            } else {
                path[i] = self.zero_hashes[i - 1];
            }

            subtree_offset += subtree_width;
        }

        path
    }

    pub fn update_leaf(&mut self, index: u32, value: Field) {
        assert(
            index < SUBTREE_ITEMS,
            "index must be less than the number of leaves in the subtree",
        );

        self.subtree.update_leaf(index, value, [0; SUBTREE_HEIGHT]);

        self.left_supertree_branch[0] = merkle_hash(
            self.subtree.get_root(),
            self.zero_hashes[SUBTREE_HEIGHT - 1],
        );
        for i in 1..self.left_supertree_branch.len() {
            self.left_supertree_branch[i] = merkle_hash(
                self.left_supertree_branch[i - 1],
                self.zero_hashes[SUBTREE_HEIGHT - 1 + i],
            );
        }
    }

    pub fn get_root(self) -> Field {
        self.left_supertree_branch[SUPERTREE_HEIGHT - 1]
    }

    // TODO make this actually track non empty leaves
    pub fn get_next_available_index(_self: Self) -> u32 {
        SUBTREE_ITEMS
    }
}

#[test]
fn test_merkle_tree_empty_subtree() {
    let tree = NonEmptyMerkleTree::new([0; 2], [0; 2], [0; 1], [0; 1]);
    assert_eq(tree.zero_hashes.len(), 2);
    let path = tree.get_sibling_path(3);
    assert_eq(path[0], 0);
    assert_eq(path[1], merkle_hash(0, 0));
    assert_eq(tree.get_root(), calculate_empty_tree_root(2));
}

#[test]
fn test_merkle_tree_empty_subtree_height_3() {
    let tree = NonEmptyMerkleTree::new([0; 2], [0; 3], [0; 2], [0; 1]);
    assert_eq(tree.zero_hashes.len(), 3);
    assert_eq(tree.get_root(), calculate_empty_tree_root(3));

    let zero_hashes = compute_zero_hashes([0; 3]);
    let path = tree.get_sibling_path(0);
    assert_eq(path[0], 0);
    assert_eq(path[1], zero_hashes[0]);
    assert_eq(path[2], zero_hashes[1]);
}

#[test]
fn test_merkle_tree_non_empty_subtree() {
    let tree = NonEmptyMerkleTree::new([1; 2], [0; 2], [0; 1], [0; 1]);
    assert_eq(tree.zero_hashes.len(), 2);
    let path = tree.get_sibling_path(3);
    assert_eq(path[0], 0);
    assert_eq(path[1], merkle_hash(1, 1));
}

#[test]
fn test_merkle_tree_non_empty_subtree_height_3() {
    let tree = NonEmptyMerkleTree::new([2, 3], [0; 3], [0; 2], [0; 1]);
    assert_eq(tree.zero_hashes.len(), 3);
    let path = tree.get_sibling_path(1);
    assert_eq(path[0], 2);
    let zero_hashes = compute_zero_hashes([0; 3]);
    assert_eq(path[1], zero_hashes[0]);
    assert_eq(path[2], zero_hashes[1]);
}

#[test]
fn test_merkle_tree_set_to_empty() {
    let mut tree = NonEmptyMerkleTree::new([2, 3], [0; 3], [0; 2], [0; 1]);
    tree.update_leaf(0, 0);
    tree.update_leaf(1, 0);
    assert_eq(tree.get_root(), calculate_empty_tree_root(3));

    let zero_hashes = compute_zero_hashes([0; 3]);
    let path = tree.get_sibling_path(0);
    assert_eq(path[0], 0);
    assert_eq(path[1], zero_hashes[0]);
    assert_eq(path[2], zero_hashes[1]);
}
