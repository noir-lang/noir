// The code below is inspired by [compute_encrypted_log](https://github.com/AztecProtocol/aztec-packages/blob/b42756bc10175fea9eb60544759e9dbe41ae5e76/noir-projects/aztec-nr/aztec/src/encrypted_logs/payload.nr#L111)
// which resulted in a bytecode size blowup when compiled to ACIR, see https://github.com/noir-lang/noir/issues/6929
// The issue was around `encrypted_bytes[offset + i]` generating large amounts of gates, as per the `flamegraph.sh` tool in aztec-packages.
// The details around encryption and addresses have been stripped away, focusing on just copying bytes of equivalent size arrays.

// Original values which resulted in huge bytecode even on this example (500K long SSA)
// global PRIVATE_LOG_SIZE_IN_FIELDS: u32 = 18;
// global ENCRYPTED_PAYLOAD_SIZE_IN_BYTES: u32 = (PRIVATE_LOG_SIZE_IN_FIELDS - 1) * 31;
// global EPH_PK_SIZE: u32 = 32;
// global HEADER_SIZE: u32 = 48;
// global OVERHEAD_PADDING: u32 = 15;

// Using the same formulas with smaller numbers; the effect is the same, but the SSA is more manageable.
global PRIVATE_LOG_SIZE_IN_FIELDS: u32 = 4;
global ENCRYPTED_PAYLOAD_SIZE_IN_BYTES: u32 = (PRIVATE_LOG_SIZE_IN_FIELDS - 1) * 5;
global EPH_PK_SIZE: u32 = 3;
global HEADER_SIZE: u32 = 2;
global OVERHEAD_PADDING: u32 = 1;

// Unused because encryption didn't play a role:
// global OVERHEAD_SIZE: u32 = EPH_PK_SIZE + HEADER_SIZE + OVERHEAD_PADDING;
// global PLAINTEXT_LENGTH_SIZE: u32 = 2;
// global MAX_PRIVATE_LOG_PLAINTEXT_SIZE_IN_BYTES: u32 =
//     ENCRYPTED_PAYLOAD_SIZE_IN_BYTES - OVERHEAD_SIZE - PLAINTEXT_LENGTH_SIZE - 1 /* aes padding */;

global BODY_SIZE: u32 =
    ENCRYPTED_PAYLOAD_SIZE_IN_BYTES - EPH_PK_SIZE - HEADER_SIZE - OVERHEAD_PADDING;

fn main(
    eph_pk_bytes: [u8; EPH_PK_SIZE],
    incoming_header_ciphertext: [u8; HEADER_SIZE],
    incoming_body_ciphertext: [u8; BODY_SIZE],
    flag: bool,
) -> pub [u8; ENCRYPTED_PAYLOAD_SIZE_IN_BYTES] {
    compute_encrypted_log(
        eph_pk_bytes,
        incoming_header_ciphertext,
        incoming_body_ciphertext,
        flag,
    )
}

fn compute_encrypted_log<let M: u32>(
    eph_pk_bytes: [u8; EPH_PK_SIZE],
    incoming_header_ciphertext: [u8; HEADER_SIZE],
    incoming_body_ciphertext: [u8; BODY_SIZE],
    flag: bool,
) -> [u8; M] {
    let mut encrypted_bytes = [0; M];
    let mut offset = 0;

    // NOTE: Adding a conditional variable can result in the array being fully copied, item by item,
    // in each iteration in the second loop that copies incoming_body_ciphertext into encrypted_bytes.
    // Depending on where we place the `flag` we either get the item-by-item copying (blowup),
    // or just a single array item gets read and a new array constructed in each iteration (no blowup).

    // If the `flag` is here then it blows up.
    if flag {
        // eph_pk
        for i in 0..EPH_PK_SIZE {
            encrypted_bytes[offset + i] = eph_pk_bytes[i];
        }
        offset += EPH_PK_SIZE;

        // If the `flag` is here then it blows up.
        // if flag {

        // incoming_header
        for i in 0..HEADER_SIZE {
            encrypted_bytes[offset + i] = incoming_header_ciphertext[i];
        }
        offset += HEADER_SIZE;

        // Padding.
        offset += OVERHEAD_PADDING;

        // If the `flag` is here then it does not blow up.
        //if flag {
        // incoming_body
        // Then we fill in the rest as the incoming body ciphertext
        let size = M - offset;

        // NOTE: This made the bytecode size blowup disappear in aztec packages,
        // but in this reproduction the size seems to be statically known regardless.
        // let size = M - 32 - HEADER_SIZE - OVERHEAD_PADDING;

        assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
        for i in 0..size {
            encrypted_bytes[offset + i] = incoming_body_ciphertext[i];
        }
    }

    encrypted_bytes
}
