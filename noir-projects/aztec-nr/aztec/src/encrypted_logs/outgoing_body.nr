use dep::protocol_types::{
    address::AztecAddress, scalar::Scalar, point::Point, constants::GENERATOR_INDEX__SYMMETRIC_KEY,
    hash::poseidon2_hash
};
use std::aes128::aes128_encrypt;

use crate::{keys::point_to_symmetric_key::point_to_symmetric_key, utils::point::point_to_bytes};

struct EncryptedLogOutgoingBody {
    eph_sk: Scalar,
    recipient: AztecAddress,
    recipient_ivpk: Point,
}

impl EncryptedLogOutgoingBody {
    pub fn new(eph_sk: Scalar, recipient: AztecAddress, recipient_ivpk: Point) -> Self {
        Self { eph_sk, recipient, recipient_ivpk }
    }

    pub fn compute_ciphertext(self, ovsk_app: Scalar, eph_pk: Point) -> [u8; 144] {
        // Again, we could compute `eph_pk` here, but we keep the interface more similar
        // and also make it easier to optimise it later as we just pass it along

        let mut buffer = [0 as u8; 128];

        let serialized_eph_sk_high = self.eph_sk.hi.to_be_bytes(32);
        let serialized_eph_sk_low = self.eph_sk.lo.to_be_bytes(32);

        let address_bytes = self.recipient.to_field().to_be_bytes(32);
        let serialized_recipient_ivpk = point_to_bytes(self.recipient_ivpk);

        for i in 0..32 {
            buffer[i] = serialized_eph_sk_high[i];
            buffer[i + 32] = serialized_eph_sk_low[i];
            buffer[i + 64] = address_bytes[i];
        }
        for i in 0..32 {
            buffer[i + 96] = serialized_recipient_ivpk[i];
        }

        // We compute the symmetric key using poseidon.
        let full_key: [u8; 32] = poseidon2_hash(
            [
            ovsk_app.hi, ovsk_app.lo, eph_pk.x, eph_pk.y,
            GENERATOR_INDEX__SYMMETRIC_KEY as Field
        ]
        ).to_be_bytes(32).as_array();

        let mut sym_key = [0; 16];
        let mut iv = [0; 16];

        for i in 0..16 {
            sym_key[i] = full_key[i];
            iv[i] = full_key[i + 16];
        }
        aes128_encrypt(buffer, iv, sym_key).as_array()
    }
}

mod test {
    use crate::encrypted_logs::outgoing_body::EncryptedLogOutgoingBody;
    use dep::protocol_types::{
        address::AztecAddress, traits::Empty, constants::GENERATOR_INDEX__NOTE_NULLIFIER,
        scalar::Scalar, point::Point, hash::poseidon2_hash
    };
    use std::embedded_curve_ops::fixed_base_scalar_mul as derive_public_key;

    use crate::context::PrivateContext;

    #[test]
    fn test_encrypted_log_outgoing_body() {
        let eph_sk = Scalar {
            lo: 0x00000000000000000000000000000000d0d302ee245dfaf2807e604eec4715fe,
            hi: 0x000000000000000000000000000000000f096b423017226a18461115fa8d34bb
        };
        let recipient_ivsk = Scalar {
            lo: 0x000000000000000000000000000000004828f8f95676ebb481df163f87fd4022,
            hi: 0x000000000000000000000000000000000f4d97c25d578f9348251a71ca17ae31
        };
        let sender_ovsk_app = Scalar {
            lo: 0x0000000000000000000000000000000074d2e28c6bc5176ac02cf7c7d36a444e,
            hi: 0x00000000000000000000000000000000089c6887cb1446d86c64e81afc78048b
        };

        let eph_pk = derive_public_key(eph_sk);
        let recipient_ivpk = derive_public_key(recipient_ivsk);

        let recipient = AztecAddress::from_field(0xdeadbeef);

        let body = EncryptedLogOutgoingBody::new(eph_sk, recipient, recipient_ivpk);

        let ciphertext = body.compute_ciphertext(sender_ovsk_app, eph_pk);

        let expected_outgoing_body_ciphertext = [
            127, 84, 96, 176, 101, 107, 236, 57, 68, 8, 53, 202, 138, 74, 186, 54, 74, 193, 245, 7, 109, 59, 218, 33, 1, 31, 205, 225, 241, 209, 64, 222, 94, 245, 4, 150, 47, 241, 187, 64, 152, 20, 102, 158, 200, 217, 213, 82, 1, 240, 170, 185, 51, 80, 27, 109, 63, 231, 235, 120, 174, 44, 133, 248, 10, 97, 60, 40, 222, 190, 147, 76, 187, 48, 91, 206, 48, 106, 56, 118, 38, 127, 82, 4, 182, 188, 44, 224, 31, 129, 47, 107, 134, 252, 20, 25, 249, 193, 215, 137, 195, 43, 98, 42, 54, 96, 254, 89, 134, 31, 103, 142, 16, 43, 92, 211, 145, 113, 217, 253, 161, 240, 121, 205, 146, 200, 168, 160, 221, 32, 229, 116, 26, 216, 86, 189, 78, 120, 10, 224, 85, 52, 40, 244
        ];

        for i in 0..expected_outgoing_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_outgoing_body_ciphertext[i]);
        }
        assert_eq(expected_outgoing_body_ciphertext.len(), ciphertext.len());
    }
}
