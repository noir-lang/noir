// TODO: remove once this module is used
#![allow(dead_code)]
use aztec_backend::barretenberg_rs::Barretenberg;
use noir_field::FieldElement;
use std::{convert::TryInto, path::Path};

// This impl should be redone in a more efficient and readable way.
// We should have a separate impl for SparseMerkle and regular merkle
// With Regular merkle we need to ensure that updates are done sequentially
//
// With sparse merkle, one can update at any index

pub(crate) type HashPath = Vec<(FieldElement, FieldElement)>;

pub fn flatten_path(path: Vec<(FieldElement, FieldElement)>) -> Vec<FieldElement> {
    path.into_iter()
        .flat_map(|(left, right)| std::iter::once(left).chain(std::iter::once(right)))
        .collect()
}

pub struct MerkleTree {
    depth: u32,
    total_size: u32,
    db: sled::Db,
    barretenberg: Barretenberg,
}

fn insert_root(db: &mut sled::Db, value: FieldElement) {
    db.insert("ROOT".as_bytes(), value.to_bytes()).unwrap();
}
fn fetch_root(db: &sled::Db) -> FieldElement {
    let value = db
        .get("ROOT".as_bytes())
        .unwrap()
        .expect("merkle root should always be present");
    FieldElement::from_be_bytes_reduce(&value.to_vec())
}
fn insert_depth(db: &mut sled::Db, value: u32) {
    db.insert("DEPTH".as_bytes(), &value.to_be_bytes()).unwrap();
}
fn fetch_depth(db: &sled::Db) -> u32 {
    let value = db
        .get("DEPTH".as_bytes())
        .unwrap()
        .expect("depth should always be present");
    u32::from_be_bytes(value.to_vec().try_into().unwrap())
}
fn insert_empty_index(db: &mut sled::Db, index: u32) {
    // First fetch the depth to see that this is less than
    let depth = fetch_depth(&db);
    let total_size = 1 << depth;
    if index > total_size {
        panic!(
            "trying to insert at index {}, but total width is {}",
            index, total_size
        )
    }
    db.insert("EMPTY".as_bytes(), &index.to_be_bytes()).unwrap();
}
fn fetch_empty_index(db: &sled::Db) -> u32 {
    let value = db
        .get("EMPTY".as_bytes())
        .unwrap()
        .expect("empty index should always be present");
    u32::from_be_bytes(value.to_vec().try_into().unwrap())
}
fn insert_preimage(db: &mut sled::Db, index: u32, value: Vec<u8>) {
    let tree = db.open_tree("preimages").unwrap();

    let index = index as u128;
    tree.insert(&index.to_be_bytes(), value).unwrap();
}

fn fetch_preimage(db: &sled::Db, index: usize) -> Vec<u8> {
    let tree = db.open_tree("preimages").unwrap();

    let index = index as u128;
    tree.get(&index.to_be_bytes())
        .unwrap()
        .map(|i_vec| i_vec.to_vec())
        .unwrap()
}
fn fetch_hash(db: &sled::Db, index: usize) -> FieldElement {
    let tree = db.open_tree("hashes").unwrap();
    let index = index as u128;

    tree.get(&index.to_be_bytes())
        .unwrap()
        .map(|i_vec| FieldElement::from_be_bytes_reduce(&i_vec.to_vec()))
        .unwrap()
}

fn insert_hash(db: &mut sled::Db, index: u32, hash: FieldElement) {
    let tree = db.open_tree("hashes").unwrap();
    let index = index as u128;

    tree.insert(&index.to_be_bytes(), hash.to_bytes()).unwrap();
}

fn find_hash_from_value(db: &sled::Db, leaf_value: &FieldElement) -> Option<u128> {
    let tree = db.open_tree("hashes").unwrap();

    for index_db_lef_hash in tree.iter() {
        let (key, db_leaf_hash) = index_db_lef_hash.unwrap();
        let index = u128::from_be_bytes(key.to_vec().try_into().unwrap());

        if db_leaf_hash.to_vec() == leaf_value.to_bytes() {
            return Some(index);
        }
    }
    None
}

impl MerkleTree {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MerkleTree {
        let barretenberg = Barretenberg::new();
        assert!(path.as_ref().exists(), "path does not exist");
        let config = sled::Config::new().path(path);

        let db = config.open().unwrap();

        let depth = fetch_depth(&db);

        let total_size = 1u32 << depth;

        MerkleTree {
            depth,
            total_size,
            barretenberg,
            db,
        }
    }
    pub fn new<P: AsRef<Path>>(depth: u32, path: P) -> MerkleTree {
        let mut barretenberg = Barretenberg::new();

        assert!((1..=20).contains(&depth)); // Why can depth != 0 and depth not more than 20?

        let config = sled::Config::new().path(path);
        let mut db = config.open().unwrap();

        let total_size = 1u32 << depth;

        let mut hashes: Vec<_> = (0..total_size * 2 - 2)
            .map(|_| FieldElement::zero())
            .collect();

        let zero_message = [0u8; 64];
        let pre_images = (0..total_size).map(|_| zero_message.to_vec());

        let mut current = hash(&zero_message);

        let mut offset = 0usize;
        let mut layer_size = total_size as usize; // XXX: On 32 bit architectures, this `as` cast may silently truncate, when total_size > 2^32?
        while offset < hashes.len() {
            for i in 0..layer_size {
                hashes[offset + i] = current;
            }
            current = compress_native(&mut barretenberg, &current, &current);

            offset += layer_size;
            layer_size /= 2;
        }
        let root = current;
        insert_root(&mut db, root);

        for (index, hash) in hashes.into_iter().enumerate() {
            insert_hash(&mut db, index as u32, hash)
        }

        for (index, image) in pre_images.into_iter().enumerate() {
            insert_preimage(&mut db, index as u32, image)
        }

        insert_depth(&mut db, depth);
        insert_empty_index(&mut db, 0);

        MerkleTree {
            depth,
            total_size,
            barretenberg,
            db,
        }
    }

    pub fn get_hash_path(&self, mut index: usize) -> HashPath {
        let mut path = HashPath::with_capacity(self.depth as usize);

        let mut offset = 0usize;
        let mut layer_size = self.total_size;
        for _ in 0..self.depth {
            index &= (!0) - 1;
            path.push((
                fetch_hash(&self.db, offset + index),
                fetch_hash(&self.db, offset + index + 1),
            ));
            offset += layer_size as usize;
            layer_size /= 2;
            index /= 2;
        }
        path
    }
    /// Updates the message at index and computes the new tree root
    pub fn update_message(&mut self, index: usize, new_message: &[u8]) -> FieldElement {
        let current = hash(new_message);

        insert_preimage(&mut self.db, index as u32, new_message.to_vec());
        self.update_leaf(index, current)
    }

    fn check_if_index_valid_and_increment(&mut self, mut index: usize) {
        // Fetch the empty index
        let empty_index = fetch_empty_index(&self.db) as usize;
        if empty_index == index {
            // increment the empty index
            index += 1;
            insert_empty_index(&mut self.db, index as u32);
        } else {
            panic!("this is an regular append-only merkle tree. Tried to insert at {}, but next empty is at {}", index, empty_index);
        }
    }

    pub fn find_index_from_leaf(&self, leaf_value: &FieldElement) -> Option<usize> {
        let index = find_hash_from_value(&self.db, leaf_value);
        index.map(|val| val as usize)
    }

    // TODO: this gets updated to be -1 on the latest barretenberg branch
    pub fn find_index_for_empty_leaf(&self) -> usize {
        let index = fetch_empty_index(&self.db);
        return index as usize;
    }

    /// Update the element at index and compute the new tree root
    pub fn update_leaf(&mut self, mut index: usize, mut current: FieldElement) -> FieldElement {
        // Note that this method does not update the list of messages [preimages]|
        // use `update_message` to do this
        self.check_if_index_valid_and_increment(index);

        let mut offset = 0usize;
        let mut layer_size = self.total_size;
        for _ in 0..self.depth {
            insert_hash(&mut self.db, (offset + index) as u32, current);

            index &= (!0) - 1;
            current = compress_native(
                &mut self.barretenberg,
                &fetch_hash(&self.db, offset + index),
                &fetch_hash(&self.db, offset + index + 1),
            );

            offset += layer_size as usize;
            layer_size /= 2;
            index /= 2;
        }

        insert_root(&mut self.db, current);
        current
    }
    /// Gets a message at `index`. This is not the leaf
    pub fn get_message_at_index(&self, index: usize) -> Vec<u8> {
        fetch_preimage(&self.db, index)
    }

    pub fn check_membership(
        hash_path: Vec<&FieldElement>,
        root: &FieldElement,
        index: &FieldElement,
        leaf: &FieldElement,
    ) -> FieldElement {
        assert!(hash_path.len() % 2 == 0);

        let mut barretenberg = Barretenberg::new();

        let mut index_bits = index.bits();
        index_bits.reverse();

        let mut current = *leaf;

        let mut is_member = true;

        let chunks = hash_path.chunks(2).enumerate();
        for (i, path_pair) in chunks {
            let path_bit = index_bits[i];

            let hash_left = path_pair[0];
            let hash_right = path_pair[1];

            let is_left = (&current == hash_left) & !path_bit;
            let is_right = (&current == hash_right) & path_bit;
            is_member &= is_left ^ is_right;
            current = compress_native(&mut barretenberg, hash_left, hash_right);
        }
        is_member &= &current == root;

        if is_member {
            FieldElement::one()
        } else {
            FieldElement::zero()
        }
    }

    pub fn root(&self) -> FieldElement {
        fetch_root(&self.db)
    }
    pub fn depth(&self) -> u32 {
        self.depth
    }
}

fn hash(message: &[u8]) -> FieldElement {
    use blake2::Digest;

    let mut hasher = blake2::Blake2s::new();
    hasher.update(message);
    let res = hasher.finalize();
    FieldElement::from_be_bytes_reduce(&res[..])
}
// XXX(FIXME) : Currently, this is very aztec specific, because this PWG does not have
// a way to deal with generic ECC operations
fn compress_native(
    barretenberg: &mut Barretenberg,
    left: &FieldElement,
    right: &FieldElement,
) -> FieldElement {
    barretenberg.compress_native(left, right)
}

#[test]
fn basic_interop_initial_root() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    // Test that the initial root is computed correctly
    let tree = MerkleTree::new(3, &temp_dir);
    // Copied from barretenberg by copying the stdout from MemoryTree
    let expected_hex = "0620374242254671503abf57d13969d41bbae97e59fa97cd7777cd683beb9eb8";
    assert_eq!(tree.root().to_hex(), expected_hex)
}
#[test]
fn basic_interop_hashpath() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    // Test that the hashpath is correct
    let tree = MerkleTree::new(3, &temp_dir);

    let path = tree.get_hash_path(0);

    let expected_hash_path = vec![
        (
            "1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0",
            "1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0",
        ),
        (
            "262e1ae3710241581182198b69a10601148ee1dd20ae638f99cde7b3ede59754",
            "262e1ae3710241581182198b69a10601148ee1dd20ae638f99cde7b3ede59754",
        ),
        (
            "0f88aa985f23258a12a78b35eab5d3d8d41d091a71113f2d4b2731d96ab78cfd",
            "0f88aa985f23258a12a78b35eab5d3d8d41d091a71113f2d4b2731d96ab78cfd",
        ),
    ];

    for (got, expected_segment) in path.into_iter().zip(expected_hash_path) {
        assert_eq!(got.0.to_hex().as_str(), expected_segment.0);
        assert_eq!(got.1.to_hex().as_str(), expected_segment.1)
    }
}

#[test]
fn basic_interop_update() {
    // Test that computing the HashPath is correct
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    let mut tree = MerkleTree::new(3, &temp_dir);

    tree.update_message(0, &vec![0; 64]);
    tree.update_message(1, &vec![1; 64]);
    tree.update_message(2, &vec![2; 64]);
    tree.update_message(3, &vec![3; 64]);
    tree.update_message(4, &vec![4; 64]);
    tree.update_message(5, &vec![5; 64]);
    tree.update_message(6, &vec![6; 64]);
    let root = tree.update_message(7, &vec![7; 64]);

    assert_eq!(
        "241fc8d893854e78dd2d427e534357fe02279f209193f0f82e13a3fd4e15375e",
        root.to_hex()
    );

    let path = tree.get_hash_path(2);

    let expected_hash_path = vec![
        (
            "06c2335d6f7acb84bbc7d0892cefebb7ca31169a89024f24814d5785e0d05324",
            "12dc36b01cbd8a6248b04e08f0ec91aa6d11a91f030b4a7b1460281859942185",
        ),
        (
            "2a57882283ba48c2e523fbf8142c0867e82cfaba2410793d3331a9c685f40790",
            "145beb7dcd00d8eee922af3fee2b002c6e56b716630752b787acfbe685769040",
        ),
        (
            "20a7f69fa7eada3e900b803301074386ef2ea8be29f3aa943eefc3654c0a94e6",
            "073b84f35922842dcf4c10596c3fe3eab7e61939120d2aca0a531e4e6fdce22b",
        ),
    ];

    for (got, expected_segment) in path.into_iter().zip(expected_hash_path) {
        assert_eq!(got.0.to_hex().as_str(), expected_segment.0);
        assert_eq!(got.1.to_hex().as_str(), expected_segment.1)
    }
}
