// Implementation of SHA-256 mapping a byte array of variable length to
// 32 bytes.
// Internal functions act on 32-bit unsigned integers for simplicity.

// Auxiliary mappings; names as in FIPS PUB 180-4
fn rotr32(a: u32, b: u32) -> u32 // 32-bit right rotation
{
    (a >> b) | (a << (32 as u32 - b))
}

fn ch(x: u32, y: u32, z: u32) -> u32
{
    (x & y) ^ ((!x) & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32
{
    (x & y) ^ (x & z) ^ (y & z)
}

fn bigma0(x: u32) -> u32
{
    rotr32(x, 2) ^ rotr32(x, 13) ^ rotr32(x, 22)
}

fn bigma1(x: u32) -> u32
{
    rotr32(x, 6) ^ rotr32(x, 11) ^ rotr32(x, 25)
}

fn sigma0(x: u32) -> u32
{
    rotr32(x, 7) ^ rotr32(x, 18) ^ (x >> 3)
}

fn sigma1(x: u32) -> u32
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
    for j in 0..64
    {
        let t1 = out_h[7] + bigma1(out_h[4]) + ch(out_h[4], out_h[5], out_h[6])
            + K[j] + w[j];
        let t2 = bigma0(out_h[0]) + maj(out_h[0], out_h[1], out_h[2]);
        out_h[7] = out_h[6];
        out_h[6] = out_h[5];
        out_h[5] = out_h[4];
        out_h[4] = out_h[3] + t1;
        out_h[3] = out_h[2];
        out_h[2] = out_h[1];
        out_h[1] = out_h[0];
        out_h[0] = t1 + t2;
    };
    
    out_h
}

// Convert 64-byte array to array of 16 u32s
fn msg_u8_to_u32(msg: [u8; 64]) -> [u32; 16]
{
    let mut msg32: [u32; 16] = [0; 16];

    for i in 0..16
    {
        for j in 0..4
        {
            msg32[15 - i] = (msg32[15 - i] << 8) + msg[64 - 4*(i + 1) + j] as u32;
        };
    };
    
    msg32
}

// SHA-256 hash function
fn digest(msg: [u8]) -> [u8; 32]
{
    let mut msg_block: [u8; 64] = [0; 64];
    let mut h: [u32; 8] = [1779033703,3144134277,1013904242,2773480762,1359893119,2600822924,528734635,1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut c: [u32; 8] = [0; 8]; // Compression of current message block as sequence of u32
    let mut out_h: [u8; 32] = [0; 32]; // Digest as sequence of bytes
    let mut i = 0; // Message byte pointer

    for k in 0..crate::array::len(msg)
    {
        // Populate msg_block
        msg_block[i] = msg[k];
        i = i + 1;
        if i == 64 // Enough to hash block
        {
            c = sha_c(msg_u8_to_u32(msg_block), h);
            for j in 0..8
            {
                h[j] = c[j] + h[j];
            };
            
            i = 0;
        }
    };

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).
    msg_block[i] = 1 << 7;
    i = i + 1;

    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if i >= 57 // Not enough bits (64) to store length. Fill up with zeros.
    {
        if i < 64
        {
            for _i in 57..64
            {
                if i <= 63
                {
                    msg_block[i] = 0;
                    i = i + 1;
                }
            };
        }
        c = h;
        c = sha_c(msg_u8_to_u32(msg_block), c);
        for j in 0..8
        {
            h[j] = c[j] + h[j];
        };
        
        i = 0;
    }

    for _i in 0..64 // In any case, fill blocks up with zeros until the last 64 (i.e. until i = 56).
    {
        if i < 56
        {
            msg_block[i] = 0;
            i = i + 1;
        } else if i < 64
            {
                let mut len = 8*crate::array::len(msg) as u64;
                for j in 0..8
                {
                    msg_block[63 - j] = len as u8;
                    len = len >> 8;
                };
                i = i + 8;
            }
    };

    // Hash final padded block
    c = h;
    c = sha_c(msg_u8_to_u32(msg_block), c);
    for j in 0..8
    {
        h[j] = c[j] + h[j];
    };

    // Return final hash as byte array
    for j in 0..8
    {
        for k in 0..4
        {
            out_h[31 - 4*j - k] = h[7 - j] as u8;
            h[7-j] = h[7-j] >> 8;
        };
    };

    out_h
}
