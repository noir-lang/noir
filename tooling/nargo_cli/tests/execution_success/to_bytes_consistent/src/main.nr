// This test aims to check that we have consistent behavior 
// between a `to_be_bytes` call (which is radix decomposition under the hood) 
// with constant inputs or with witness inputs.

// x = 2040124
fn main(x : Field) {
    let byte_array = x.to_be_bytes(31);
    let x_as_constant = 2040124;
    let constant_byte_array = x_as_constant.to_be_bytes(31);
    assert(constant_byte_array.len() == byte_array.len());
    for i in 0..constant_byte_array.len() {
        assert(constant_byte_array[i] == byte_array[i]);
    }
}
