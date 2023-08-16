
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
}

// #[test]
// fn test_vec() {
//     let vec: BoundedVec<Field, 2> = BoundedVec::new(0);
//     assert(vec.len == 0);
//     let vec1 = vec.push(1);
//     assert(vec1.len == 1);
//     let vec2 = vec1.push(1);
//     assert(vec2.len == 2);
//     let vec3 = vec2.push(1);
//     assert(vec3.len == 3);
//     let x = vec3.pop();
//     assert(x == 1);
// }