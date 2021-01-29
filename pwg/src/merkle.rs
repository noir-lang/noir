use noir_field::FieldElement;

pub type HashPath = Vec<(FieldElement, FieldElement)>;

pub struct MerkleTree{
    depth : u32,
    total_size : u32,
    root : FieldElement,
    hashes : Vec<FieldElement>,
    pre_images : Vec<Vec<u8>>, 
}

impl MerkleTree {
    pub fn new(depth : u32) -> MerkleTree {
        assert!(depth >= 1 && depth <= 20); // Why can depth != 0 and depth not more than 20?

        let total_size = 1u32 << depth;

        let mut hashes : Vec<_>= (0..total_size * 2 - 2).map(|_| FieldElement::zero()).collect();

        let zero_message = [0u8;64];
        let pre_images : Vec<Vec<u8>>= (0..total_size).map(|_| zero_message.to_vec()).collect(); 

        let mut current = hash(&zero_message);
        
        let mut offset = 0usize;
        let mut layer_size = total_size as usize; // XXX: On 32 bit architectures, this `as` cast may silently truncate, when total_size > 2^32? 
        while offset < hashes.len() {

            for i in 0..layer_size {
                hashes[offset + i] = current;
            }
            current = compress_native(current, current);

            offset += layer_size;
            layer_size = layer_size / 2;
        }
        let root = current;

        MerkleTree {
            depth, 
            total_size,
            root,
            pre_images,
            hashes
        }
    }

    pub fn get_hash_path(&self, mut index : usize) -> HashPath {
        let mut path = HashPath::with_capacity(self.depth as usize);

        let mut offset = 0usize;
        let mut layer_size = self.total_size;
        for _ in 0..self.depth {
            index &= (!0) - 1;
            path.push((self.hashes[offset + index], self.hashes[offset + index + 1]));
            offset += layer_size as usize;
            layer_size /= 2;
            index /= 2;
        }
        path
    }
    /// Updates the message at index and computes the new tree root
    pub fn update_message(&mut self,mut index : usize, new_message : Vec<u8>) -> FieldElement {
        let current = hash(&new_message);
        self.pre_images[index] = new_message;
        self.update_leaf(index, current)

    }
    /// Update the element at index and compute the new tree root
    pub fn update_leaf(&mut self,mut index : usize, mut current : FieldElement) -> FieldElement {

        // Note that this method does not update the list of messages [preimages]|
        // use `update_message` to do this

        let mut offset = 0usize;
        let mut layer_size = self.total_size;
        for _ in 0..self.depth {
            self.hashes[offset + index] = current;
            index &= (!0) - 1;
            current = compress_native(self.hashes[offset + index], self.hashes[offset + index + 1]);
            
            offset += layer_size as usize;
            layer_size /= 2;
            index /= 2;
        }
        self.root = current;

        self.root
    }
    /// Gets a message at `index`. This is not the leaf
    pub fn get_message_at_index(&self,index :usize ) -> Vec<u8> {
        self.pre_images[index].clone()
    }
}

fn hash(message : &[u8]) -> FieldElement {
    aztec_backend::barretenberg_rs::blake2s::hash_to_field(message)
}

// This is what the blake2s hash to function should look like
// if it was all in  Rust. This function panics because `from_bytes` 
// requires that the input is canonical and reduced.
// blake2s will produce output that is 256 bits while 
// bn254 is 254 bits
// fn hash(message : &[u8]) -> FieldElement {
    // let mut hasher = Blake2s::new();
    // hasher.update(message);
    // let res = hasher.finalize();
    // FieldElement::from_bytes(&res[..])
// }

// XXX(FIXME) : Currently, this is very aztec specific, because this PWG does not have
// a way to deal with generic ECC operations
fn compress_native(left : FieldElement, right : FieldElement) -> FieldElement {
    aztec_backend::barretenberg_rs::pedersen::compress_native(left, right)
}

#[test]
fn basic_interop_initial_root() {
    // Test that the initial root is computed correctly
    let tree = MerkleTree::new(3);
    // Copied from barretenberg by copying the stdout from MemoryTree
    let expected_hex = "0620374242254671503abf57d13969d41bbae97e59fa97cd7777cd683beb9eb8";
    assert_eq!(tree.root.to_hex(), expected_hex)
}
#[test]
fn basic_interop_hashpath() {
    // Test that the hashpath is correct
    let tree = MerkleTree::new(3);

    let path = tree.get_hash_path(0);

    let expected_hash_path = vec![
        ("1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0","1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0" ),
        ("262e1ae3710241581182198b69a10601148ee1dd20ae638f99cde7b3ede59754","262e1ae3710241581182198b69a10601148ee1dd20ae638f99cde7b3ede59754" ),
        ("0f88aa985f23258a12a78b35eab5d3d8d41d091a71113f2d4b2731d96ab78cfd","0f88aa985f23258a12a78b35eab5d3d8d41d091a71113f2d4b2731d96ab78cfd" ),
    ];

    for (got, expected_segment) in path.into_iter().zip(expected_hash_path) {
        assert_eq!(got.0.to_hex().as_str(), expected_segment.0);
        assert_eq!(got.1.to_hex().as_str(), expected_segment.1)
    }
}
#[test]
fn basic_interop_update() {
    // Test that the hashpath is correct
    let mut tree = MerkleTree::new(3);

    tree.update_message(0, vec![0;64]);
    tree.update_message(1, vec![1;64]);
    tree.update_message(2, vec![2;64]);
    tree.update_message(3, vec![3;64]);
    tree.update_message(4, vec![4;64]);
    tree.update_message(5, vec![5;64]);
    tree.update_message(6, vec![6;64]);
    let root = tree.update_message(7, vec![7;64]);

    assert_eq!("241fc8d893854e78dd2d427e534357fe02279f209193f0f82e13a3fd4e15375e", root.to_hex());

    let path = tree.get_hash_path(2);

    let expected_hash_path = vec![
        ("06c2335d6f7acb84bbc7d0892cefebb7ca31169a89024f24814d5785e0d05324","12dc36b01cbd8a6248b04e08f0ec91aa6d11a91f030b4a7b1460281859942185"),
        ("2a57882283ba48c2e523fbf8142c0867e82cfaba2410793d3331a9c685f40790","145beb7dcd00d8eee922af3fee2b002c6e56b716630752b787acfbe685769040"),
        ("20a7f69fa7eada3e900b803301074386ef2ea8be29f3aa943eefc3654c0a94e6","073b84f35922842dcf4c10596c3fe3eab7e61939120d2aca0a531e4e6fdce22b"),
    ];

    for (got, expected_segment) in path.into_iter().zip(expected_hash_path) {
        assert_eq!(got.0.to_hex().as_str(), expected_segment.0);
        assert_eq!(got.1.to_hex().as_str(), expected_segment.1)
    }
}