use dep::protocol_types::{
    address::AztecAddress, grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint,
    constants::GENERATOR_INDEX__SYMMETRIC_KEY, hash::poseidon2_hash
};

use dep::std::aes128::aes128_encrypt;
use dep::std::println;

use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogOutgoingBody {
    eph_sk: GrumpkinPrivateKey,
    recipient: AztecAddress,
    recipient_ivpk_app: GrumpkinPoint,
}

impl EncryptedLogOutgoingBody {
    pub fn new(
        eph_sk: GrumpkinPrivateKey,
        recipient: AztecAddress,
        recipient_ivpk_app: GrumpkinPoint
    ) -> Self {
        Self { eph_sk, recipient, recipient_ivpk_app }
    }

    pub fn compute_ciphertext(self, ovsk_app: GrumpkinPrivateKey, eph_pk: GrumpkinPoint) -> [u8; 176] {
        // Again, we could compute `eph_pk` here, but we keep the interface more similar
        // and also make it easier to optimise it later as we just pass it along

        let mut buffer: [u8; 160] = [0; 160];

        let serialized_eph_sk: [Field; 2] = self.eph_sk.serialize();
        let serialized_eph_sk_high = serialized_eph_sk[0].to_be_bytes(32);
        let serialized_eph_sk_low = serialized_eph_sk[1].to_be_bytes(32);

        let address_bytes = self.recipient.to_field().to_be_bytes(32);
        let serialized_recipient_ivpk_app = self.recipient_ivpk_app.serialize();
        let serialized_recipient_ivpk_app_x = serialized_recipient_ivpk_app[0].to_be_bytes(32);
        let serialized_recipient_ivpk_app_y = serialized_recipient_ivpk_app[1].to_be_bytes(32);

        for i in 0..32 {
            buffer[i] = serialized_eph_sk_high[i];
            buffer[i + 32] = serialized_eph_sk_low[i];
            buffer[i + 64] = address_bytes[i];
            buffer[i + 96] = serialized_recipient_ivpk_app_x[i];
            buffer[i + 128] = serialized_recipient_ivpk_app_y[i];
        }

        // We compute the symmetric key using poseidon.
        let full_key: [u8; 32] = poseidon2_hash(
            [
            ovsk_app.high, ovsk_app.low, eph_pk.x, eph_pk.y,
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
        grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint, hash::poseidon2_hash
    };

    use crate::context::PrivateContext;

    #[test]
    fn test_encrypted_log_outgoing_body() {
        let eph_sk = GrumpkinPrivateKey::new(
            0x000000000000000000000000000000000f096b423017226a18461115fa8d34bb,
            0x00000000000000000000000000000000d0d302ee245dfaf2807e604eec4715fe
        );
        let recipient_ivsk_app = GrumpkinPrivateKey::new(
            0x000000000000000000000000000000000f4d97c25d578f9348251a71ca17ae31,
            0x000000000000000000000000000000004828f8f95676ebb481df163f87fd4022
        );
        let sender_ovsk_app = GrumpkinPrivateKey::new(
            0x00000000000000000000000000000000089c6887cb1446d86c64e81afc78048b,
            0x0000000000000000000000000000000074d2e28c6bc5176ac02cf7c7d36a444e
        );

        let eph_pk = eph_sk.derive_public_key();
        let recipient_ivpk_app = recipient_ivsk_app.derive_public_key();

        let recipient = AztecAddress::from_field(0xdeadbeef);

        let body = EncryptedLogOutgoingBody::new(eph_sk, recipient, recipient_ivpk_app);

        let ciphertext = body.compute_ciphertext(sender_ovsk_app, eph_pk);

        let expected_outgoing_body_ciphertext = [
            126, 10, 214, 39, 130, 143, 96, 143, 79, 143, 22, 36, 55, 41, 234, 255, 226, 26, 138, 236, 91, 188, 204, 216, 172, 133, 134, 69, 161, 237, 134, 5, 75, 192, 10, 6, 229, 54, 194, 56, 103, 243, 57, 248, 147, 237, 4, 3, 39, 28, 226, 30, 237, 228, 212, 115, 246, 244, 105, 39, 129, 119, 126, 207, 176, 14, 75, 134, 241, 23, 2, 187, 239, 86, 47, 56, 239, 20, 92, 176, 70, 12, 219, 226, 150, 70, 192, 43, 125, 53, 230, 153, 135, 228, 210, 197, 76, 123, 185, 190, 61, 172, 29, 168, 241, 191, 205, 71, 136, 72, 52, 115, 232, 246, 87, 42, 50, 150, 134, 108, 225, 90, 191, 191, 182, 150, 124, 147, 78, 249, 144, 111, 122, 187, 187, 5, 249, 167, 186, 14, 228, 128, 158, 138, 55, 99, 228, 46, 219, 187, 248, 122, 70, 31, 39, 209, 127, 23, 244, 84, 14, 93, 86, 208, 155, 151, 238, 70, 63, 3, 137, 59, 206, 230, 4, 20
        ];

        for i in 0..expected_outgoing_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_outgoing_body_ciphertext[i]);
        }
        assert_eq(expected_outgoing_body_ciphertext.len(), ciphertext.len());
    }
}
