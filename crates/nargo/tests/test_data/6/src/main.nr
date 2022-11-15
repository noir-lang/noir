// Sha256 circuit where the input is 5 bytes
// not five field elements since sha256 operates over
// bytes.
//
// If you do not cast, it will take all the bytes from the field element!

// Mimc input is an array of field elements
// The function is called mimc_bn254 to emphasize its parameters are chosen for bn254 curve, it should be used only with a proving system using the same curve (e.g Plonk from Aztec)
use dep::std;

fn main(x: [u8; 5], result: pub [u8; 32]) {
    let mut digest = std::hash::sha256(x);
    digest[0] = 5 as u8;
    digest = std::hash::sha256(x);
    constrain digest == result;

    let y = [12,45,78,41];
    let h = std::hash::mimc_bn254(y);
   constrain h == 18226366069841799622585958305961373004333097209608110160936134895615261821931;
}
