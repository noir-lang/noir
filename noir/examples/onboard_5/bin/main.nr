// This is the sha256, but we are using slices instead of "0xbd"

use dep::std;

fn main(x : [1]u8, low : Public, _high: Public) {

     let digest = std::hash::sha256(x);
     
     constrain digest[0] == low;
}