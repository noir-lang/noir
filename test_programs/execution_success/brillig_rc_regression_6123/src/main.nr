struct Builder {
    note_hashes: BoundedVec<Field, 2>,
    nullifiers: BoundedVec<Field, 2>,
}

impl Builder {
    fn append_note_hashes_with_logs(&mut self, num_note_hashes: u32) {
        let index_offset = self.note_hashes.len();
        for i in 0..self.note_hashes.max_len() {
            if i < num_note_hashes {
                self.add_new_note_hash((index_offset + i) as Field);
            }
        }
    }

    fn add_new_note_hash(&mut self, value: Field) {
        self.note_hashes.push(value);
    }
}

fn swap_items<T, let N: u32>(vec: &mut BoundedVec<T, N>, from_index: u32, to_index: u32) {
    let tmp = vec.storage[from_index];
    vec.storage[from_index] = vec.storage[to_index];
    vec.storage[to_index] = tmp;
}

unconstrained fn main() {
    let mut builder = Builder { note_hashes: BoundedVec::new(), nullifiers: BoundedVec::new() };

    builder.append_note_hashes_with_logs(2);
    builder.nullifiers.storage[1] = 27;
    // Get ordered items before shuffling.
    let note_hashes = builder.note_hashes.storage;
    let original_first_note_hash = note_hashes[0];
    // Shuffle.
    swap_items(&mut builder.note_hashes, 1, 0);

    for i in 0..1 {
        assert_eq(note_hashes[i], original_first_note_hash);
    }
}
