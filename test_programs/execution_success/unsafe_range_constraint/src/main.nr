// Test that we can apply a range constraint to a field using 
// a builtin.
fn main(x: field) {
    x.assert_max_bit_size(48);
}
