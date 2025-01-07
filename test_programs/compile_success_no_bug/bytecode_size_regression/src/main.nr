// The code below is inspired by [compute_encrypted_log](https://github.com/AztecProtocol/aztec-packages/blob/b42756bc10175fea9eb60544759e9dbe41ae5e76/noir-projects/aztec-nr/aztec/src/encrypted_logs/payload.nr#L111)
// which resulted in a bytecode size blowup when compiled to ACIR, see https://github.com/noir-lang/noir/issues/6929
// The issue was around `encrypted_bytes[offset + i]` generating large amounts of gates, as per the `flamegraph.sh` tool in aztec-packages.
// The details around encryption and addresses have been stripped away, focusing on just copying bytes of equivalent size arrays.

use std::aes128::aes128_encrypt;

global PRIVATE_LOG_SIZE_IN_FIELDS: u32 = 18;
global ENCRYPTED_PAYLOAD_SIZE_IN_BYTES: u32 = (PRIVATE_LOG_SIZE_IN_FIELDS - 1) * 31;
global EPH_PK_SIZE: u32 = 32;
global HEADER_SIZE: u32 = 48;
global OVERHEAD_PADDING: u32 = 15;
global OVERHEAD_SIZE: u32 = EPH_PK_SIZE + HEADER_SIZE + OVERHEAD_PADDING;
global PLAINTEXT_LENGTH_SIZE: u32 = 2;
global MAX_PRIVATE_LOG_PLAINTEXT_SIZE_IN_BYTES: u32 =
    ENCRYPTED_PAYLOAD_SIZE_IN_BYTES - OVERHEAD_SIZE - PLAINTEXT_LENGTH_SIZE - 1 /* aes padding */;

fn main(
    eph_pk_bytes: [u8; EPH_PK_SIZE],
    aes_secret: [u8; 32],
    incoming_header_ciphertext: [u8; HEADER_SIZE],
    extended_plaintext: [u8; MAX_PRIVATE_LOG_PLAINTEXT_SIZE_IN_BYTES + PLAINTEXT_LENGTH_SIZE],
    shift: u32,
) -> pub [u8; ENCRYPTED_PAYLOAD_SIZE_IN_BYTES] {
    // unsafe {
    //     compute_encrypted_log_unconstrained(
    //         eph_pk_bytes,
    //         aes_secret,
    //         incoming_header_ciphertext,
    //         extended_plaintext,
    //         shift,
    //     )
    // }
    compute_encrypted_log(
        eph_pk_bytes,
        aes_secret,
        incoming_header_ciphertext,
        extended_plaintext,
        shift,
    )
}

unconstrained fn compute_encrypted_log_unconstrained<let P: u32, let M: u32>(
    eph_pk_bytes: [u8; EPH_PK_SIZE],
    aes_secret: [u8; 32],
    incoming_header_ciphertext: [u8; HEADER_SIZE],
    plaintext: [u8; MAX_PRIVATE_LOG_PLAINTEXT_SIZE_IN_BYTES + PLAINTEXT_LENGTH_SIZE],
    shift: u32,
) -> [u8; M] {
    compute_encrypted_log(
        eph_pk_bytes,
        aes_secret,
        incoming_header_ciphertext,
        plaintext,
        shift,
    )
}

fn compute_encrypted_log<let P: u32, let M: u32>(
    eph_pk_bytes: [u8; EPH_PK_SIZE],
    aes_secret: [u8; 32],
    incoming_header_ciphertext: [u8; HEADER_SIZE],
    plaintext: [u8; P],
    shift: u32,
) -> [u8; M] {
    let mut encrypted_bytes = [0; M];
    let mut offset = 0;

    // eph_pk
    for i in 0..EPH_PK_SIZE {
        encrypted_bytes[offset + i] = eph_pk_bytes[i];
    }
    offset += EPH_PK_SIZE;

    // incoming_header
    for i in 0..HEADER_SIZE {
        encrypted_bytes[offset + i] = incoming_header_ciphertext[i];
    }
    offset += HEADER_SIZE;

    // Padding.
    offset += OVERHEAD_PADDING;

    // incoming_body
    let incoming_body_ciphertext = compute_incoming_body_ciphertext(plaintext, aes_secret);
    // Then we fill in the rest as the incoming body ciphertext
    let size = M - offset;

    // NOTE: This made the bytecode size blowup disappear in aztec packages,
    // but in this reproduction the size seems to be statically known regardless.
    //let size = M - 32 - HEADER_SIZE - OVERHEAD_PADDING;

    assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
    for i in 0..size {
        // NOTE: Adding `shift` makes the index dynamic enough to force ACIR to use memory operations.
        // Without `shift` it assigns directly to witnesses, which is good, but not what we wanted to reproduce.
        encrypted_bytes[offset + i + shift] = incoming_body_ciphertext[i];
    }

    encrypted_bytes
}

pub fn compute_incoming_body_ciphertext<let P: u32>(
    plaintext: [u8; P],
    aes_secret: [u8; 32],
) -> [u8] {
    let full_key = aes_secret;
    let mut sym_key = [0; 16];
    let mut iv = [0; 16];

    for i in 0..16 {
        sym_key[i] = full_key[i];
        iv[i] = full_key[i + 16];
    }
    aes128_encrypt(plaintext, iv, sym_key)
}
