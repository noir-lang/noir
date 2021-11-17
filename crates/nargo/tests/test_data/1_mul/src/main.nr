// Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
// The circuit should handle properly the growth of the bit size

fn main(x : u32, y : u32, z : u32) {
    x = x * y;
    x = x * x;  //144
    x = x * x;  //20736
    x = x * x;  //429 981 696
    constrain (x) == z;
}