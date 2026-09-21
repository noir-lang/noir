// Every element of this array occupies no cell, so the array has no memory to check an index
// against: the check is the whole of the access.
fn main(i: u32, x: Field) -> pub Field {
    let mut arr: [[Field; 0]; 4] = [[], [], [], []];
    arr[i] = [];
    x
}
