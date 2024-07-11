use dep::protocol_types::{address::AztecAddress, scalar::Scalar, point::Point};

use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

use std::aes128::aes128_encrypt;

struct EncryptedLogHeader {
    address: AztecAddress,
}

impl EncryptedLogHeader {
    fn new(address: AztecAddress) -> Self {
        EncryptedLogHeader { address }
    }

    fn compute_ciphertext(self, secret: Scalar, point: Point) -> [u8; 48] {
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
    let secret = Scalar {
        lo: 0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd,
        hi: 0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06
    };
    let point = Point {
        x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
        y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
        is_infinite: false
    };

    let ciphertext = header.compute_ciphertext(secret, point);

    let expected_header_ciphertext = [
        228, 9, 65, 81, 62, 59, 249, 207, 90, 196, 206, 72, 39, 199, 82, 196, 23, 131, 32, 226, 26, 176, 43, 39, 239, 177, 177, 192, 85, 216, 17, 15, 18, 187, 35, 225, 135, 192, 63, 88, 29, 173, 232, 46, 72, 82, 187, 139
    ];

    assert_eq(ciphertext, expected_header_ciphertext);
}
