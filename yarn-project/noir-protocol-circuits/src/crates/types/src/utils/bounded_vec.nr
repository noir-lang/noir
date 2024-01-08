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

    pub fn push_array<Len>(&mut self, array: [T; Len]) {
        let new_len = self.len + array.len();
        assert(new_len as u64 <= MaxLen as u64, "push_array out of bounds");
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = new_len;
    }

    pub fn push_vec<Len>(&mut self, vec: BoundedVec<T, Len>) {
        let append_len = vec.len();
        let new_len = self.len + append_len;
        assert(new_len as u64 <= MaxLen as u64, "push_vec out of bounds");

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

#[test]
fn test_vec_push_pop() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new(0);
    assert(vec.len == 0);
    vec.push(2);
    assert(vec.len == 1);
    vec.push(4);
    assert(vec.len == 2);
    vec.push(6);
    assert(vec.len == 3);
    let x = vec.pop();
    assert(x == 6);
    assert(vec.len == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
}

#[test]
fn test_vec_push_array() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new(0);
    vec.push_array([2, 4]);
    assert(vec.len == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
}

#[test(should_fail_with="push_array out of bounds")]
fn test_vec_push_array_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    vec.push_array([2, 4, 6]);
}

#[test(should_fail_with="push_array out of bounds")]
fn test_vec_push_array_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    vec.push_array([2]);
    assert(vec.len == 1);
    vec.push_array([4, 6]);
}

#[test(should_fail)]
fn test_vec_get_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    vec.push_array([2, 4]);
    let _x = vec.get(2);
}

#[test(should_fail)]
fn test_vec_get_not_declared() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    vec.push_array([2]);
    let _x = vec.get(1);
}

#[test(should_fail)]
fn test_vec_get_uninitialized() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    let _x = vec.get(0);
}

#[test(should_fail_with="push out of bounds")]
fn test_vec_push_out_of_bound() {
    let mut vec: BoundedVec<Field, 1> = BoundedVec::new(0);
    vec.push(1);
    vec.push(2);
}

#[test(should_fail_with="push_vec out of bounds")]
fn test_vec_push_vec_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);

    let mut another_vec: BoundedVec<Field, 3> = BoundedVec::new(0);
    another_vec.push_array([1, 2, 3]);

    vec.push_vec(another_vec);
}

#[test(should_fail_with="push_vec out of bounds")]
fn test_vec_push_vec_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new(0);
    vec.push_array([1, 2]);

    let mut another_vec: BoundedVec<Field, 1> = BoundedVec::new(0);
    another_vec.push(3);

    vec.push_vec(another_vec);
}

#[test]
fn test_vec_any() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new(0);
    vec.push_array([2, 4, 6]);
    assert(vec.any(|v| v == 2) == true);
    assert(vec.any(|v| v == 4) == true);
    assert(vec.any(|v| v == 6) == true);
    assert(vec.any(|v| v == 3) == false);
}

#[test]
fn test_vec_any_not_default() {
    let default_value = 1;
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new(default_value);
    vec.push_array([2, 4]);
    assert(vec.any(|v| v == default_value) == false);
}
