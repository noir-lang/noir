struct BoundedVec<T, MaxLen> {
    storage: [T; MaxLen],
    // TODO: change this to return a u64 as Noir now
    // uses u64 for indexing
    len: Field,
    empty_value: T,
}

impl<T, MaxLen> BoundedVec<T, MaxLen> {
    pub fn new(initial_value: T) -> Self {
        BoundedVec { storage: [initial_value; MaxLen], len: 0, empty_value: initial_value }
    }

    pub fn get(mut self: Self, index: Field) -> T {
        assert(index as u64 < self.len as u64);
        self.storage[index]
    }

    pub fn get_unchecked(mut self: Self, index: Field) -> T {
        self.storage[index]
    }

    pub fn push(&mut self, elem: T) {
        assert(self.len as u64 < MaxLen as u64, "push out of bounds");

        self.storage[self.len] = elem;
        self.len += 1;
    }

    pub fn len(self) -> Field {
        self.len
    }

    pub fn max_len(_self: BoundedVec<T, MaxLen>) -> Field {
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

    pub fn extend_from_bounded_vec<Len>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len as u64 <= MaxLen as u64, "extend_from_bounded_vec out of bounds");

        let mut exceeded_len = false;
        for i in 0..Len {
            exceeded_len |= i == append_len;
            if !exceeded_len {
                self.storage[self.len + (i as Field)] = vec.get_unchecked(i as Field);
            }
        }
        self.len = new_len;
    }

    pub fn pop(&mut self) -> T {
        assert(self.len as u64 > 0);
        self.len -= 1;

        let elem = self.storage[self.len];
        self.storage[self.len] = self.empty_value;
        elem
    }

    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        let mut exceeded_len = false;
        for i in 0..MaxLen {
            exceeded_len |= i == self.len;
            if (!exceeded_len) {
                ret |= predicate(self.storage[i]);
            }
        }
        ret
    }
}