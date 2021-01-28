use noir_field::FieldElement;


pub type HashPath = Vec<(FieldElement, FieldElement)>;
struct MerkleTree{
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
    /// Update the element at index and compute the new tree root
    pub fn update_element(&mut self,mut index : usize, new_message : Vec<u8>) -> FieldElement {
        let mut current = hash(&new_message);
        self.pre_images[index] = new_message;

        let mut offset = 0usize;
        let mut layer_size = self.total_size;
        for i in 0..self.depth {
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
    pub fn get_message_at_index(&self,index :usize ) -> Vec<u8> {
        self.pre_images[index].clone()
    }
}

fn hash(message : &[u8]) -> FieldElement {
    todo!()
}

fn compress_native(left : FieldElement, right : FieldElement) -> FieldElement {
    todo!()
}