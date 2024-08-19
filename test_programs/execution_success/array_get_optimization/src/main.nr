global SIZE = 500;

fn main(mut fields: [Field; SIZE], enables: [bool; SIZE], indices: [u64; SIZE]) -> pub [Field; SIZE] {
    for i in 0..SIZE {
        if enables[i] {
            fields[indices[i]] = i as Field;
        }
    }

    fields
}
