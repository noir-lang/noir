// Adversarial case for collapsing the conditional array-set chain: a struct-of-`Field`
// array that is deliberately *mostly discarded* — only `arr[0].a` is read at the end.
// Each conditional forms a chain of array_sets over the flattened array
// (offsets +0/+2/+4 within the element):
//   arr1 = array_set arr0, offset+0, val_a
//   arr2 = array_set arr1, offset+2, val_c
//   arr3 = array_set arr2, offset+4, val_e
// Because `Field` arithmetic is side-effect free, the per-element branch merges for the
// discarded elements are dead code and get eliminated, leaving a small circuit. Merging
// the chain into explicit predicated array_sets keeps work for those elements instead,
// so for this discard-heavy `Field` shape it costs more than it saves. The `_u64` twin
// is the opposite: integer side effects keep the merges live regardless, so collapsing
// the chain there is a clear win.
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
