use dep::protocol_types::{address::AztecAddress, grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint};

use crate::oracle::encryption::aes128_encrypt;
use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogHeader {
    address: AztecAddress,
}

impl EncryptedLogHeader {
    fn new(address: AztecAddress) -> Self {
        EncryptedLogHeader { address }
    }

    // @todo Issue(#5901) Figure out if we return the bytes or fields for the log
    fn compute_ciphertext(self, secret: GrumpkinPrivateKey, point: GrumpkinPoint) -> [u8; 32] {
        let full_key = point_to_symmetric_key(secret, point);
        let mut sym_key = [0; 16];
        let mut iv = [0; 16];
        let mut input = [0; 32];
        let input_slice = self.address.to_field().to_be_bytes(32);

        for i in 0..16 {
            sym_key[i] = full_key[i];
            iv[i] = full_key[i + 16];

            // We copy address on the following 2 lines in order to avoid having 2 loops  
            input[i] = input_slice[i];
            input[i + 16] = input_slice[i + 16];
        }

        // @todo Issue(#6172) This encryption is currently using an oracle. It is not actually constrained atm.
        aes128_encrypt(input, iv, sym_key)
    }
}

// @todo Issue(#6172) This is to be run as a test. But it is currently using the AES oracle so will fail there.
fn test_encrypted_log_header() {
    let address = AztecAddress::from_field(0xdeadbeef);
    let header = EncryptedLogHeader::new(address);
    let secret = GrumpkinPrivateKey::new(
        0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06,
        0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd
    );
    let point = GrumpkinPoint::new(
        0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
        0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e
    );

    let ciphertext = header.compute_ciphertext(secret, point);

    let expected_header_ciphertext = [
        131, 119, 105, 129, 244, 32, 151, 205, 12, 99, 93, 62, 10, 180, 72, 21, 179, 36, 250, 95, 56, 167, 171, 16, 195, 164, 223, 57, 75, 5, 24, 119
    ];

    assert_eq(ciphertext, expected_header_ciphertext);
}
