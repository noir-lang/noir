trait Empty {
    fn empty() -> Self;
}

impl Empty for Field {
    fn empty() -> Self {
        0
    }
}

pub fn is_empty<T>(item: T) -> bool where T: Empty + Eq {
    item.eq(T::empty())
}

pub fn array_to_bounded_vec<T, let N: u32>(array: [T; N]) -> BoundedVec<T, N> where T: Empty + Eq {
    let mut len = 0;
    for elem in array {
        if !is_empty(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len }
}

global TX_SIZE: u32 = 5;
global APP_CALL_SIZE: u32 = 2;

fn main(
    a: call_data(0) [Field; TX_SIZE],
    b: call_data(1) [Field; APP_CALL_SIZE]
) -> return_data [Field; TX_SIZE] {
    let mut a_as_bounded_vec = array_to_bounded_vec(a);

    for i in 0..APP_CALL_SIZE {
        let value = b[i];
        if value != 0 {
            a_as_bounded_vec.push(value);
        }
    }
    a_as_bounded_vec.storage()
}
