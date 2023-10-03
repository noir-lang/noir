// Test accessing a multidimensional array
fn main(x: Field, y: Field, array_input: [[Field; 2]; 2]) {
    assert(array_input[0][0] == x);
    assert(array_input[0][1] == y);

    let arr: [[Field; 2]; 1] = [[3,3]];
    assert_eq(arr[0], array_input[1]);
}
