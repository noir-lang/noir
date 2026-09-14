// Same multi-field conditional modification as `large_nested_array_multi_field_merge`,
// but over `u64` members instead of `Field`. Each conditional forms a chain of
// array_sets over the flattened array (offsets +0/+2/+4 within the element):
//   arr1 = array_set arr0, offset+0, val_a
//   arr2 = array_set arr1, offset+2, val_c
//   arr3 = array_set arr2, offset+4, val_e
// Integer arithmetic carries truncation/range-check side effects, so the element-wise
// branch merges for elements that are never read again are NOT dead code and cannot be
// eliminated (unlike the `Field` twin). This is the shape that benefits most from
// collapsing the chain into O(chain_length) merged array_sets instead of an
// O(array_size) element-wise IfElse merge per iteration.
struct Element {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
}

fn main(input: [Element; 64], condition: bool) {
    let mut arr = input;
    for i in 0..64 {
        if condition {
            arr[i].a = arr[i].a + 1;
            arr[i].c = arr[i].c + 2;
            arr[i].e = arr[i].e + 3;
        }
    }
    assert(arr[0].a > 0);
}
