// This program will fail when there is a specific difference between the two fields
#[fuzz]
fn comparison(a: Field, b: Field) {
    let c = b + 0xffffffffffff0000000000;
    let d = c - 0xeeeeeeeeeeeeeeeee;
    assert((a !=d) | (b == 0) | (a == 0));
}
