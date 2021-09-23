use std::path::PathBuf;

use acvm::backends::csat_3_plonk_aztec::pwg::merkle::MerkleTree;
use std::path::Path;

fn main() {
    let mut path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    path.pop();
    path.push("data/merkle_db");

    let mut merkle_tree = MerkleTree::new(3, path);

    let root = merkle_tree.update_message(1, &[0, 0, 1, 2, 3]);
    dbg!(root.to_hex());
}
