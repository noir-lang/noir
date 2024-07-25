use crate::{cmp::Eq, convert::From};

struct BoundedVec<T, let MaxLen: u32> {
    storage: [T; MaxLen],
    len: u32,
}

impl<T, let MaxLen: u32> BoundedVec<T, MaxLen> {
    pub fn new() -> Self {
        let zeroed = crate::unsafe::zeroed();
        BoundedVec { storage: [zeroed; MaxLen], len: 0 }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn get(self, index: u32) -> T {
        assert(index < self.len, "Attempted to read past end of BoundedVec");
        self.get_unchecked(index)
    }

    /// Get an element from the vector at the given index.
    /// Responds with undefined data for `index` where `self.len < index < self.max_len()`.
    pub fn get_unchecked(self, index: u32) -> T {
        self.storage[index]
    }

    /// Write an element to the vector at the given index.
    /// Panics if the given index points beyond the end of the vector (`self.len()`).
    pub fn set(&mut self, index: u32, value: T) {
        assert(index < self.len, "Attempted to write past end of BoundedVec");
        self.set_unchecked(index, value)
    }

    /// Write an element to the vector at the given index.
    /// Does not check whether the passed `index` is a valid index within the vector.
    ///
    /// Silently writes past the end of the vector for `index` where `self.len < index < self.max_len()`
    /// Panics if the given index points beyond the maximum length of the vector (`self.max_len()`).
    pub fn set_unchecked(&mut self, index: u32, value: T) {
        self.storage[index] = value;
    }

    pub fn push(&mut self, elem: T) {
        assert(self.len < MaxLen, "push out of bounds");

        self.storage[self.len] = elem;
        self.len += 1;
    }

    pub fn len(self) -> u32 {
        self.len
    }

    pub fn max_len(_self: BoundedVec<T, MaxLen>) -> u32 {
        MaxLen
    }

    // This is a intermediate method, while we don't have an
    // .extend method
    pub fn storage(self) -> [T; MaxLen] {
        self.storage
    }

    pub fn extend_from_array<let Len: u32>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();
        assert(new_len <= MaxLen, "extend_from_array out of bounds");
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_slice(&mut self, slice: [T]) {
        let new_len = self.len + slice.len();
        assert(new_len <= MaxLen, "extend_from_slice out of bounds");
        for i in 0..slice.len() {
            self.storage[self.len + i] = slice[i];
        }
        self.len = new_len;
    }

    pub fn extend_from_bounded_vec<let Len: u32>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len <= MaxLen, "extend_from_bounded_vec out of bounds");

        let mut exceeded_len = false;
        for i in 0..Len {
            exceeded_len |= i == append_len;
            if !exceeded_len {
                self.storage[self.len + i] = vec.get_unchecked(i);
            }
        }
        self.len = new_len;
    }

    pub fn from_array<let Len: u32>(array: [T; Len]) -> Self {
        assert(Len <= MaxLen, "from array out of bounds");
        let mut vec: BoundedVec<T, MaxLen> = BoundedVec::new();
        vec.extend_from_array(array);
        vec
    }

    pub fn pop(&mut self) -> T {
        assert(self.len > 0);
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

    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> BoundedVec<U, MaxLen> {
        let mut ret = BoundedVec::new();
        ret.len = self.len();
        for i in 0..MaxLen {
            if i < self.len() {
                ret.storage[i] = f(self.get_unchecked(i));
            }
        }
        ret
    }
}

impl<T, let MaxLen: u32> Eq for BoundedVec<T, MaxLen> where T: Eq {
    fn eq(self, other: BoundedVec<T, MaxLen>) -> bool {
        // TODO: https://github.com/noir-lang/noir/issues/4837
        //
        // We make the assumption that the user has used the proper interface for working with `BoundedVec`s
        // rather than directly manipulating the internal fields as this can result in an inconsistent internal state.
        
        (self.len == other.len) & (self.storage == other.storage)
    }
}

impl<T, let MaxLen: u32, let Len: u32> From<[T; Len]> for BoundedVec<T, MaxLen> {
    fn from(array: [T; Len]) -> BoundedVec<T, MaxLen>  {
        BoundedVec::from_array(array)
    }
}
