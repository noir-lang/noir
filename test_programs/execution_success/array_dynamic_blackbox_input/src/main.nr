fn main(leaf: [u8; 32], path: [u8; 64], index: u32, root: [u8; 32]) {
    compute_root(leaf, path, index, root);
}

fn compute_root(leaf: [u8; 32], path: [u8; 64], _index: u32, root: [u8; 32]) {
    let mut current = leaf;
    let mut index = _index;

    for i in 0..2 {
        let mut hash_input = [0; 64];
        let offset = i * 32;
        let is_right = (index & 1) != 0;
        let a = if is_right { 32 } else { 0 };
        let b = if is_right { 0 } else { 32 };

        for j in 0..32 {
            hash_input[j + a] = current[j];
            hash_input[j + b] = path[offset + j];
        }

        current = dep::std::hash::sha256(hash_input);
        index = index >> 1;
    }

    // Regression for issue #4258
    assert(root == current);
}