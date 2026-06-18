// Large array of structs, conditionally modifying several `Field` members per loop
// iteration. Each conditional forms a chain of array_sets over the flattened array
// (offsets +0/+2/+4 within the element):
//   arr1 = array_set arr0, offset+0, val_a
//   arr2 = array_set arr1, offset+2, val_c
//   arr3 = array_set arr2, offset+4, val_e
// Because `Field` arithmetic is side-effect free, the element-wise branch merges for
// elements that are never read again are pure dead code and get eliminated, leaving
// only the live element. This case stays small regardless of how the array_set chain
// is handled. The `_u64` twin is where the element type changes the calculation:
// integer side effects keep those merges live, so collapsing the chain matters there.
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
