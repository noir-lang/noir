fn main(mut x: u32, y: pub u32, z: u32) {
    x *= y;
    x *= x; //144
    x *= x; //20736
    x *= x; //429 981 696
    assert(x == z);
}