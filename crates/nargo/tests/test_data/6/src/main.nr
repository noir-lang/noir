// Sha256 circuit where the input is 5 bytes
// not five field elements since sha256 operates over
// bytes.
//
// If you do not cast, it will take all the bytes from the field element!
use dep::std;

fn main(x: [u8; 5], result: pub [u8; 32]) {
    let mut digest = std::hash::sha256(x);
    digest[0] = 5 as u8;
    digest = std::hash::sha256(x);
    constrain digest == result;
}
