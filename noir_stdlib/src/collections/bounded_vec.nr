use crate::cmp::Eq;

struct BoundedVec<T, MaxLen> {
    storage: [T; MaxLen],
    len: u64,
}

impl<T, MaxLen> BoundedVec<T, MaxLen> {
    pub fn new() -> Self {
        let zeroed = crate::unsafe::zeroed();
        BoundedVec { storage: [zeroed; MaxLen], len: 0 }
    }

    pub fn get(mut self: Self, index: u64) -> T {
        assert(index as u64 < self.len);
        self.storage[index]
    }

    pub fn get_unchecked(mut self: Self, index: u64) -> T {
        self.storage[index]
    }

    pub fn push(&mut self, elem: T) {
        assert(self.len < MaxLen as u64, "push out of bounds");

        self.storage[self.len] = elem;
        self.len += 1;
    }

    pub fn len(self) -> u64 {
        self.len
    }

    pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u64 {
        MaxLen
    }

    // This is a intermediate method, while we don't have an
    // .extend method
    pub fn storage(self) -> [T; MaxLen] {
        self.storage
    }

    pub fn extend_from_array<Len>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();
        assert(new_len as u64 <= MaxLen as u64, "extend_from_array out of bounds");
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_slice(&mut self, slice: [T]) {
        let new_len = self.len + slice.len();
        assert(new_len as u64 <= MaxLen as u64, "extend_from_slice out of bounds");
        for i in 0..slice.len() {
            self.storage[self.len + i] = slice[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_bounded_vec<Len>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len as u64 <= MaxLen as u64, "extend_from_bounded_vec out of bounds");

        let mut exceeded_len = false;
        for i in 0..Len {
            exceeded_len |= i == append_len;
            if !exceeded_len {
                self.storage[self.len + i] = vec.get_unchecked(i);
            }
        }
        self.len = new_len;
    }

    pub fn pop(&mut self) -> T {
        assert(self.len as u64 > 0);
        self.len -= 1;

        let elem = self.storage[self.len];
        self.storage[self.len] = crate::unsafe::zeroed();
        elem
    }

    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        let mut exceeded_len = false;
        for i in 0..MaxLen {
            exceeded_len |= i == self.len;
            if !exceeded_len {
                ret |= predicate(self.storage[i]);
            }
        }
        ret
    }
}

impl<T, MaxLen> Eq for BoundedVec<T, MaxLen> where T: Eq {
    fn eq(self, other: BoundedVec<T, MaxLen>) -> bool {
        let equal_lens = self.len == other.len;

        let mut ret = true;
        let mut exceeded_len = false;
        for i in 0..MaxLen {
            exceeded_len |= i == self.len;
            if !exceeded_len {
                ret &= (self.storage[i] == other.storage[i]);
            }
        }
        equal_lens & ret
    }
}

mod bounded_vec_tests {
    // TODO: Allow imports from "super"
    use crate::collections::bounded_vec::BoundedVec;

    #[test]
    fn empty_equality() {
        let mut bounded_vec1: BoundedVec<Field, 3> = BoundedVec::new();
        let mut bounded_vec2: BoundedVec<Field, 3> = BoundedVec::new();

        assert_eq(bounded_vec1, bounded_vec2);
    }

    #[test]
    fn inequality() {
        let mut bounded_vec1: BoundedVec<Field, 3> = BoundedVec::new();
        let mut bounded_vec2: BoundedVec<Field, 3> = BoundedVec::new();
        bounded_vec1.push(1);
        bounded_vec2.push(2);

        assert(bounded_vec1 != bounded_vec2);
    }

    #[test]
    fn equality_respects_specified_length() {
        let mut bounded_vec1: BoundedVec<Field, 3> = BoundedVec::new();
        bounded_vec1.push(1);

        // This BoundedVec has an extra value past the end of its specified length,
        // this should be ignored when checking equality so they are considered equal.
        let mut bounded_vec2: BoundedVec<Field, 3> = BoundedVec { storage: [1, 2, 0], len: 1 };

        assert_eq(bounded_vec1, bounded_vec2);

        // Pushing another entry onto `bounded_vec1` to make the underlying arrays equal should
        // result in the `BoundedVec`s being unequal as their lengths are different.
        bounded_vec1.push(2);

        assert(bounded_vec1 != bounded_vec2);

        bounded_vec2.push(2);

        assert_eq(bounded_vec1, bounded_vec2);
    }
}
