use dep::std;

// Testing signed integer division: 
//  7/3  = 2
// -7/3  = -2
// -7/-3 = 2
//  7/-3 = -2
fn main(mut x: i32, mut y: i32, mut z: i32) {
    // 7/3 = 2
    assert(x / y == z);

    // -7/3 = -2
    x = 0-x;
    z = 0-z;
    assert(x+7==0);
    assert(z+2==0);
    assert(x / y == z);

    // -7/-3 = 2
    y = 0 - y;
    z = 0-z;
    assert(x / y == z);
   
    // 7/-3 = -2
    x = 0 - x;
    z = 0-z;
    assert(x / y == z);
}
