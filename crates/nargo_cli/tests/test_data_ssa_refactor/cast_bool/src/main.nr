fn main(x: Field, y: Field) {
    let z = x == y;
    let t = z as u8;
    assert(t == 1);
}
    
