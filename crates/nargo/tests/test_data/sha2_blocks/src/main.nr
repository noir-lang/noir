// Test Noir implementations of SHA256 and SHA512 on one- and two-block (padded) messages.
use dep::std;

fn main(x: [u8; 3], result256: [u8; 32], result512: [u8; 64])
{
	// One-block tests.
	let mut digest256 = std::sha256::digest(x);
	constrain digest256 == result256;

	let mut digest512 = std::sha512::digest(x);
	constrain digest512 == result512;

	// Two-block SHA256 test. Taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf
	let y: [u8; 56] = [97,98,99,100,98,99,100,101,99,100,101,102,100,101,102,103,101,102,103,104,102,103,104,105,103,104,105,106,104,105,106,107,105,106,107,108,106,107,108,109,107,108,109,110,108,109,110,111,109,110,111,112,110,111,112,113]; // "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
	digest256 = std::sha256::digest(y);
	constrain digest256 == [36,141,106,97,210,6,56,184,229,192,38,147,12,62,96,57,163,60,228,89,100,255,33,103,246,236,237,212,25,219,6,193];

	// Two-block SHA256 test. Taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA512.pdf
	let z: [u8; 112] = [97,98,99,100,101,102,103,104,98,99,100,101,102,103,104,105,99,100,101,102,103,104,105,106,100,101,102,103,104,105,106,107,101,102,103,104,105,106,107,108,102,103,104,105,106,107,108,109,103,104,105,106,107,108,109,110,104,105,106,107,108,109,110,111,105,106,107,108,109,110,111,112,106,107,108,109,110,111,112,113,107,108,109,110,111,112,113,114,108,109,110,111,112,113,114,115,109,110,111,112,113,114,115,116,110,111,112,113,114,115,116,117]; // "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
	digest512 = std::sha512::digest(z);
	constrain digest512 == [142,149,155,117,218,227,19,218,140,244,247,40,20,252,20,63,143,119,121,198,235,159,127,161,114,153,174,173,182,136,144,24,80,29,40,158,73,0,247,228,51,27,153,222,196,181,67,58,199,211,41,238,182,221,38,84,94,150,229,91,135,75,233,9];
}
