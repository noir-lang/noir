// Test Noir implementations of SHA256 and SHA512 on a one-byte message.
use dep::std;

fn main(x: Field, result256: [u8; 32], result512: [u8; 64])
{
	let digest256 = std::sha256::digest([x as u8]);
	constrain digest256 == result256;

	let digest512 = std::sha512::digest([x as u8]);
	constrain digest512 == result512;
}
