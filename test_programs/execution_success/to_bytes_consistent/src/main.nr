// This test aims to check that we have consistent behavior
// between a `to_be_bytes` call (which is radix decomposition under the hood)
// with constant inputs or with witness inputs.
// x = 2040124
fn main(x: Field) {
    let byte_array: [u8; 31] = x.to_be_bytes();
    let x_as_constant = 2040124;
    let constant_byte_array: [u8; 31] = x_as_constant.to_be_bytes();
    assert(constant_byte_array.len() == byte_array.len());
    for i in 0..constant_byte_array.len() {
        assert(constant_byte_array[i] == byte_array[i]);
    }
}
