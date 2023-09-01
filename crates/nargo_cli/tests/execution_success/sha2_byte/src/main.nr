// Test Noir implementations of SHA256 and SHA512 on a one-byte message.
use dep::std;

fn main(x: Field, result256: [u8; 32], result512: [u8; 64])
{
	// let digest256 = std::sha256::digest([x as u8]);
	// assert(digest256 == result256);

	// let digest512 = std::sha512::digest([x as u8]);
	// assert(digest512 == result512);


	let mut msg_block: [u8; 64] = [0; 64];
	msg_block[0] = x as u8;
	let msg = msg_u8_to_u32(msg_block);
    let mut h: [u32; 8] = [1779033703,3144134277,1013904242,2773480762,1359893119,2600822924,528734635,1541459225]; // Intermediate hash, starting with the canonical initial value

	let x = sha_c(msg, h);
}

// Convert 64-byte array to array of 16 u32s
fn msg_u8_to_u32(msg: [u8; 64]) -> [u32; 16]
{
    let mut msg32: [u32; 16] = [0; 16];
    // std::println(msg32[15] << 1);

    for i in 0..16
    {

        for j in 0..4
        {
            msg32[15 - i] = (msg32[15 - i] << 8) + msg[64 - 4*(i + 1) + j] as u32;
        };
    };
    
    msg32
}

fn rotr32(a: u32, b: u32) -> u64 // 32-bit right rotation
{
	dep::std::println(f"a: {a}");
	dep::std::println(f"b: {b}");
	// let q = a <<
    (a >> b) | (a << (32 as u32 - b))
	// a
}

fn ch(x: u32, y: u32, z: u32) -> u32
{
    (x & y) ^ ((!x) & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32
{
    (x & y) ^ (x & z) ^ (y & z)
}

fn bigma0(x: u32) -> u64
{
    rotr32(x, 2) ^ rotr32(x, 13) ^ rotr32(x, 22)
}

fn bigma1(x: u32) -> u64
{
    rotr32(x, 6) ^ rotr32(x, 11) ^ rotr32(x, 25)
}

fn sigma0(x: u32) -> u64
{
    rotr32(x, 7) ^ rotr32(x, 18) ^ (x >> 3)
}

fn sigma1(x: u32) -> u64
{
    rotr32(x, 17) ^ rotr32(x, 19) ^ (x >> 10)
}

fn sha_w(msg: [u32; 16]) -> [u32; 64] // Expanded message blocks
{

    let mut w: [u32;64] = [0; 64];
    
    for j in 0..16
    {
        w[j] = msg[j];
    };

    for j in 16..64
    {
        w[j] = sigma1(w[j-2]) + w[j-7] + sigma0(w[j-15]) + w[j-16];
    };
    
    w
}

// SHA-256 compression function
fn sha_c(msg: [u32; 16], hash: [u32; 8]) -> [u32; 8]
{

    let K: [u32; 64] = [1116352408,1899447441,3049323471,3921009573,961987163,1508970993,2453635748,2870763221,3624381080,310598401,607225278,1426881987,1925078388,2162078206,2614888103,3248222580,3835390401,4022224774,264347078,604807628,770255983,1249150122,1555081692,1996064986,2554220882,2821834349,2952996808,3210313671,3336571891,3584528711,113926993,338241895,666307205,773529912,1294757372,1396182291,1695183700,1986661051,2177026350,2456956037,2730485921,2820302411,3259730800,3345764771,3516065817,3600352804,4094571909,275423344,430227734,506948616,659060556,883997877,958139571,1322822218,1537002063,1747873779,1955562222,2024104815,2227730452,2361852424,2428436474,2756734187,3204031479,3329325298]; // first 32 bits of fractional parts of cube roots of first 64 primes
    
    let mut out_h: [u32; 8] = hash;
    let w = sha_w(msg);
    // let w: [u32; 64] = [0; 64];
    for j in 0..64
    {
        let t1 = out_h[7] + bigma1(out_h[4]) + ch(out_h[4], out_h[5], out_h[6])
            + K[j] + w[j];
        let t2 = bigma0(out_h[0]) as u64 + maj(out_h[0], out_h[1], out_h[2]) as u64;
        out_h[7] = out_h[6];
        out_h[6] = out_h[5];
        out_h[5] = out_h[4];
		dep::std::println(out_h[3]);
		dep::std::println(t1);
        out_h[4] = out_h[3] + t1;
        out_h[3] = out_h[2];
        out_h[2] = out_h[1];
        out_h[1] = out_h[0];
        out_h[0] = t1 + t2 as u32;
    };
	// let mut new_out_h: [u32; 8] = [0; 8];
    // for i in 0..8 {
	// 	new_out_h[i] = out_h[i] as u32;
	// }
    // new_out_h
	out_h
}
