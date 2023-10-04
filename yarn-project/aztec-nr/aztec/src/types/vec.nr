
struct BoundedVec<T, MaxLen> {
    storage: [T; MaxLen],
    len: Field,
}

impl<T, MaxLen> BoundedVec<T, MaxLen> {
    fn new(initial_value: T) -> Self {
        BoundedVec { storage: [initial_value; MaxLen], len: 0 }
    }

    fn get(mut self: Self, index: Field) -> T {
        assert(index as u64 < self.len as u64);
        self.storage[index]
    }

    fn get_unchecked(mut self: Self, index: Field) -> T {
        self.storage[index]
    }

    fn push(&mut self, elem: T) {
        assert(self.len as u64 < MaxLen as u64);

        self.storage[self.len] = elem;
        self.len += 1;
    }

    fn push_array<Len>(&mut self, array: [T; Len]) {
        let newLen = self.len + array.len();
        assert(newLen as u64 <= MaxLen as u64);
        for i in 0..array.len() {
            self.storage[self.len + i] = array[i];
        }
        self.len = newLen;
    }

    fn pop(&mut self) -> T {
        assert(self.len as u64 > 0);

        let elem = self.storage[self.len - 1];
        self.len -= 1;
        elem
    }

    fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
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

#[test(should_fail)]
fn test_vec_push_overflow() {
    let mut vec: BoundedVec<Field, 1> = BoundedVec::new(0);
    vec.push(1);
    vec.push(2);
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