// Implementation of SHA-512 mapping a byte array of variable length to
// 64 bytes.
// Internal functions act on 64-bit unsigned integers for simplicity.

// Auxiliary mappings; names as in FIPS PUB 180-4
fn rotr64(a: u64, b: u64) -> u64 // 64-bit right rotation
{
    (a >> b) | (a << (64 - b))
}

fn sha_ch(x: u64, y: u64, z: u64) -> u64
{
    (x & y) ^ (!x & z)
}

fn sha_maj(x: u64, y: u64, z: u64) -> u64
{
    (x & y) ^ (x & z) ^ (y & z)
}

fn sha_bigma0(x: u64) -> u64
{
    rotr64(x, 28) ^ rotr64(x, 34) ^ rotr64(x, 39)
}

fn sha_bigma1(x: u64) -> u64
{
    rotr64(x, 14) ^ rotr64(x, 18) ^ rotr64(x, 41)
}

fn sha_sigma0(x: u64) -> u64
{
    rotr64(x, 1) ^ rotr64(x, 8) ^ (x >> 7)
}

fn sha_sigma1(x: u64) -> u64
{
    rotr64(x, 19) ^ rotr64(x, 61) ^ (x >> 6)
}

fn sha_w(msg: [u64; 16]) -> [u64; 80] // Expanded message blocks
{

    let mut w: [u64;80] = [0; 80];
    
    for j in 0..16
    {
        w[j] = msg[j];
    };

    for j in 16..80
    {
        w[j] = sha_sigma1(w[j-2]) + w[j-7] + sha_sigma0(w[j-15]) + w[j-16];
    };
    
    w
}

// SHA-512 compression function
fn sha_c(msg: [u64; 16], hash: [u64; 8]) -> [u64; 8]
{
    let K: [u64; 80] = [4794697086780616226, 8158064640168781261, 13096744586834688815, 16840607885511220156, 4131703408338449720, 6480981068601479193, 10538285296894168987, 12329834152419229976, 15566598209576043074, 1334009975649890238, 2608012711638119052, 6128411473006802146, 8268148722764581231, 9286055187155687089, 11230858885718282805, 13951009754708518548, 16472876342353939154, 17275323862435702243, 1135362057144423861, 2597628984639134821, 3308224258029322869, 5365058923640841347, 6679025012923562964, 8573033837759648693, 10970295158949994411, 12119686244451234320, 12683024718118986047, 13788192230050041572, 14330467153632333762, 15395433587784984357, 489312712824947311, 1452737877330783856, 2861767655752347644, 3322285676063803686, 5560940570517711597, 5996557281743188959, 7280758554555802590, 8532644243296465576, 9350256976987008742, 10552545826968843579, 11727347734174303076, 12113106623233404929, 14000437183269869457, 14369950271660146224, 15101387698204529176, 15463397548674623760, 17586052441742319658, 1182934255886127544, 1847814050463011016, 2177327727835720531, 2830643537854262169, 3796741975233480872, 4115178125766777443, 5681478168544905931, 6601373596472566643, 7507060721942968483, 8399075790359081724, 8693463985226723168, 9568029438360202098, 10144078919501101548, 10430055236837252648, 11840083180663258601, 13761210420658862357, 14299343276471374635, 14566680578165727644, 15097957966210449927, 16922976911328602910, 17689382322260857208, 500013540394364858, 748580250866718886, 1242879168328830382, 1977374033974150939, 2944078676154940804, 3659926193048069267, 4368137639120453308, 4836135668995329356, 5532061633213252278, 6448918945643986474, 6902733635092675308, 7801388544844847127]; // first 64 bits of fractional parts of cube roots of first 80 primes
    
    let mut out_h: [u64; 8] = hash;
    let w = sha_w(msg);
    for j in 0..80
    {
        let t1 = out_h[7] + sha_bigma1(out_h[4]) + sha_ch(out_h[4], out_h[5], out_h[6])
            + K[j] + w[j];
        let t2 = sha_bigma0(out_h[0]) + sha_maj(out_h[0], out_h[1], out_h[2]);
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

// Convert 128-byte array to array of 16 u64s
fn msg_u8_to_u64(msg: [u8; 128]) -> [u64; 16]
{
    let mut msg64: [u64; 16] = [0; 16];

    for i in 0..16
    {
        for j in 0..8
        {
            msg64[15 - i] = (msg64[15 - i] << 8) + msg[128 - 8*(i + 1) + j] as u64;
        };
    };
    
    msg64
}

// SHA-512 hash function
fn digest(msg: [u8]) -> [u8; 64]
{
    let mut msg_block: [u8; 128] = [0; 128];
    let mut h: [u64; 8] = [7640891576956012808, 13503953896175478587, 4354685564936845355, 11912009170470909681, 5840696475078001361, 11170449401992604703, 2270897969802886507, 6620516959819538809]; // Intermediate hash, starting with the canonical initial value
    let mut c: [u64; 8] = [0; 8]; // Compression of current message block as sequence of u64
    let mut out_h: [u8; 64] = [0; 64]; // Digest as sequence of bytes
    let mut i = 0; // Message byte pointer

    for m in msg
    {
        // Populate msg_block
        msg_block[i] = m;
        i = i + 1;
        if i == 128 // Enough to hash block
        {
            c = sha_c(msg_u8_to_u64(msg_block), h);
            for j in 0..8
            {
                h[j] = c[j] + h[j];
            };
            
            i = 0;
        }
    };

    // Pad the rest such that we have a [u64; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).
    msg_block[i] = 1 << 7;
    i = i + 1;
    
    // If i >= 113, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if i >= 113 // Not enough bits (128) to store length. Fill up with zeros.
    {

        if i < 128
        {
            for _i in 113..128
            {
                if i <= 127
                {
                    msg_block[i] = 0;
                    i = i + 1;
                }
            };
        }
        c = sha_c(msg_u8_to_u64(msg_block), h);
        for j in 0..8
        {
            h[j] = c[j] + h[j];
        };
        
        i = 0;
    }

    for _i in 0..128 // In any case, fill blocks up with zeros until the last 128 (i.e. until i = 112).
    {
        if i < 112
        {
            msg_block[i] = 0;
            i = i + 1;
        } else if i < 128
        {
            let mut len = 8*crate::array::len(msg) as u64; // u128 unsupported
            for j in 0..16
                {
                    msg_block[127 - j] = len as u8;
                    len = len >> 8;
                };
                i = i + 16; // Done.
            }
    };

    // Hash final padded block
    c = sha_c(msg_u8_to_u64(msg_block), h);
    for j in 0..8
    {
        h[j] = c[j] + h[j];
    };

    // Return final hash as byte array
    for j in 0..8
    {
        for k in 0..8
        {
            out_h[63 - 8*j - k] = h[7 - j] as u8;
            h[7-j] = h[7-j] >> 8;
        };
    };

    out_h
}
