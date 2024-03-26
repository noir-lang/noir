// this code is not cpp
// game recognize game
//
// assert!("airdrop eligible")

use dep::std::hash::poseidon2::Poseidon2;

use dep::std::ec::tecurve::affine::{Point,Curve};
use dep::std::ec::consts::te::{baby_jubjub};

global bjj = baby_jubjub();
global G = bjj.base8;

// Compute the public key X = x*G
fn derive_pubkey(x: Field) -> (Point) {
    let X = bjj.curve.mul(x, G);
    X
}


// encrypt msg of size 32 bytes using pubkey and randomness r
fn encrypt(pubkey: Point, r: Field, msg: [Field; 32]) -> (Point, [Field; 32]) {
    // gA = pubkey
    let gR  = derive_pubkey(r);
    let gAR = bjj.curve.mul(r, pubkey);

    let mut cipherText = [0;32];

    for i in 0..msg.len() {
        let m = msg[i];

        // hash(m || counter) 
        let hashKey = Poseidon2::hash([gAR.x, i as Field], 2);

        cipherText[i] = m + hashKey;
    }

    (gR, cipherText)
}

fn decrypt(privkey: Field, cipher: (Point, [Field;32])) -> [Field; 32] {

    let (gR, cipherText) = cipher;
    let gAR = bjj.curve.mul(privkey, gR); 

    let mut clearText = [0;32];

    for i in 0..cipherText.len() {
        let m = cipherText[i];

        // hash(m || counter) 
        let hashKey = Poseidon2::hash([gAR.x, i as Field], 2);

        clearText[i] = m - hashKey;
    }

    clearText
}

fn main(pubkey: Point, r: Field, msg: [Field; 32]) -> pub (Point, [Field; 32]) {
    encrypt(pubkey, r, msg)
}


#[test]
fn test_main() {
    let a = 0x3fbbccb240537392421955b07a0d65eded9e7637995bf2f9cfe29e19b580e4;
    let r = 0x106885ee5c8f2757c6bde259b31b3f00300f538c8901b28c5bcf982c85e17493;
    let mut msg = [0; 32];
    msg[0] = 42;
    msg[1] = 137; // alpha baby
    msg[2] = 118; 

    let gA = derive_pubkey(a);
    println(gA);

    let (gR, cipherText) = encrypt(gA, r, msg);
    let clearText = decrypt(a, (gR, cipherText));

    assert(msg == clearText);
}
