// Test accessing a multidimensional array
fn main(x: Field, y: Field) {
    let arr: [[Field; 2]; 1] = [[1, 2]];
    assert(arr[0][0] == x);
    assert(arr[0][1] == y);
}
