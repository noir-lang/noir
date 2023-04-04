
struct BoundedVec<T, MaxLen> {
    storage: [T; MaxLen],
    len: comptime Field,
}

impl<T, MaxLen> BoundedVec<T, MaxLen> {
    fn new(initial_value: T) -> Self {
        BoundedVec { storage: [initial_value; MaxLen], len: 0 }
    }

    fn get(mut self: Self, index: comptime Field) -> T {
        constrain index as u64 < self.len as u64;
        self.storage[index]
    }

    fn push(mut self: Self, elem: T) -> Self {
        constrain self.len < MaxLen;

        self.storage[self.len] = elem;
        self.len += 1;
        self
    }

    fn pop(mut self: Self) -> (Self, T) {
        constrain self.len > 0;

        let elem = self.storage[self.len - 1];
        self.len -= 1;
        (self, elem)
    }
}

// #[test]
// fn test_vec() {
//     let vec: BoundedVec<Field, 2> = BoundedVec::new(0);
//     constrain vec.len == 0;
//     let vec1 = vec.push(1);
//     constrain vec1.len == 1;
//     let vec2 = vec1.push(1);
//     constrain vec2.len == 2;
//     let vec3 = vec2.push(1);
//     constrain vec3.len == 3;
//     let x = vec3.pop();
//     constrain x == 1;
// }