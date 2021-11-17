// Testing integer division: 7/3 = 2

fn main(x : u32, y : u32, z : u32) {
    x = x / y;
    constrain (x) == z;
}