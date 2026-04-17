// Test: Large nested array with multi-field conditional modification in a loop.
// This exercises the chain-of-array_sets pattern where modifying several fields
// of a struct inside a conditional creates a chain:
//   arr1 = array_set arr0, offset+0, val_a
//   arr2 = array_set arr1, offset+2, val_c
//   arr3 = array_set arr2, offset+4, val_e
// The optimization walks backwards through the chain to emit O(chain_length)
// merged array_sets instead of an O(array_size) IfElse merge.
struct Element {
    a: Field,
    b: Field,
    c: Field,
    d: Field,
    e: Field,
}

fn main(input: [Element; 256], condition: bool) {
    let mut arr = input;
    for i in 0..256 {
        if condition {
            arr[i].a = arr[i].a + 1;
            arr[i].c = arr[i].c + 2;
            arr[i].e = arr[i].e + 3;
        }
    }
    assert(arr[0].a != 0);
}
