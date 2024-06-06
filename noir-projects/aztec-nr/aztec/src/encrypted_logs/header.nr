use dep::protocol_types::{address::AztecAddress, grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint};

use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

use dep::std::aes128::aes128_encrypt;

struct EncryptedLogHeader {
    address: AztecAddress,
}

impl EncryptedLogHeader {
    fn new(address: AztecAddress) -> Self {
        EncryptedLogHeader { address }
    }

    fn compute_ciphertext(self, secret: GrumpkinPrivateKey, point: GrumpkinPoint) -> [u8; 48] {
        let full_key = point_to_symmetric_key(secret, point);
        let mut sym_key = [0; 16];
        let mut iv = [0; 16];

        for i in 0..16 {
            sym_key[i] = full_key[i];
            iv[i] = full_key[i + 16];
        }

        let input: [u8; 32] = self.address.to_field().to_be_bytes(32).as_array();
        aes128_encrypt(input, iv, sym_key).as_array()
    }
}

#[test]
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
        131, 119, 105, 129, 244, 32, 151, 205, 12, 99, 93, 62, 10, 180, 72, 21, 179, 36, 250, 95, 56, 167, 171, 16, 195, 164, 223, 57, 75, 5, 24, 119, 198, 34, 99, 189, 193, 183, 227, 43, 79, 204, 214, 89, 221, 153, 246, 64
    ];

    assert_eq(ciphertext, expected_header_ciphertext);
}
