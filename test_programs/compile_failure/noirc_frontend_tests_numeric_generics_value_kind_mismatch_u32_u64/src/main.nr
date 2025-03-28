
    struct BoundedVec<T, let MaxLen: u32> {
        storage: [T; MaxLen],
        // can't be compared to MaxLen: u32
        // can't be used to index self.storage
        len: u64,
    }

    impl<T, let MaxLen: u32> BoundedVec<T, MaxLen> {
        pub fn extend_from_bounded_vec<let Len: u32>(&mut self, _vec: BoundedVec<T, Len>) {
            // We do this to avoid an unused variable warning on `self`
            let _ = self.len;
            for _ in 0..Len { }
        }

        pub fn push(&mut self, elem: T) {
            assert(self.len < MaxLen, "push out of bounds");
            self.storage[self.len] = elem;
            self.len += 1;
        }
    }

    fn main() {
        let _ = BoundedVec { storage: [1], len: 1 }; // silence never constructed warning
    }
    