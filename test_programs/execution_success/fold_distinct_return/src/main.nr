fn main(x: u32, y: pub u32) {
    let new_field = new_field_in_array([x, y, 3]);
    assert(new_field[0] == 25);
}

#[fold]
fn new_field_in_array(mut input: [u32; 3]) -> [u32; 3] {
    input[0] = input[0] + 20;
    input
}
