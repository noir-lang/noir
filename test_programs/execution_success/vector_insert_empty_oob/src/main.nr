// Similar to vector_insert_oob, but without assertions that can cause false negatives,
// ie. this circuit should fail for only one reason: if inserting into an empty array is rejected.
pub fn main(b: bool, i: u32, v: u32) {
    if b {
        let mut arr = @[];
        arr = arr.insert(i, v);
        println(arr);
    }
}
