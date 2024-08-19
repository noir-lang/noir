// ACIR opcodes should now increase linearly per each increase in SIZE:
// | SIZE | ACIR Opcodes | Difference
// | 2    | 45           |
// | 3    | 67           | +22
// | 4    | 89           | +22
// | 5    | 111          | +22
// ...
// | 500  | 11001        |
global SIZE = 500;

fn main(mut fields: [Field; SIZE], enables: [bool; SIZE], indices: [u64; SIZE]) -> pub [Field; SIZE] {
    for i in 0..SIZE {
        if enables[i] {
            fields[indices[i]] = i as Field;
        }
    }

    fields
}
