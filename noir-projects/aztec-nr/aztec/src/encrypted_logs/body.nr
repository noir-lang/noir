use crate::note::{note_interface::NoteInterface};
use dep::protocol_types::{grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint};

use dep::std::aes128::aes128_encrypt_slice;
use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogBody<Note> {
    storage_slot: Field,
    note_type_id: Field,
    note: Note,
}

impl<Note> EncryptedLogBody<Note> {
    pub fn new<N>(
        storage_slot: Field,
        note_type_id: Field,
        note: Note
    ) -> Self where Note: NoteInterface<N> {
        Self { storage_slot, note_type_id, note }
    }

    pub fn compute_ciphertext<N>(
        self,
        secret: GrumpkinPrivateKey,
        point: GrumpkinPoint
    ) -> [u8] where Note: NoteInterface<N> {
        let serialized_note: [Field; N] = self.note.serialize_content();

        let mut buffer_slice: [u8] = &[];

        let storage_slot_bytes = self.storage_slot.to_be_bytes(32);
        let note_type_id_bytes = self.note_type_id.to_be_bytes(32);

        for i in 0..32 {
            buffer_slice = buffer_slice.push_back(storage_slot_bytes[i]);
        }

        for i in 0..32 {
            buffer_slice = buffer_slice.push_back(note_type_id_bytes[i]);
        }

        for i in 0..serialized_note.len() {
            let bytes = serialized_note[i].to_be_bytes(32);
            for j in 0..32 {
                buffer_slice = buffer_slice.push_back(bytes[j]);
            }
        }

        let full_key = point_to_symmetric_key(secret, point);
        let mut sym_key = [0; 16];
        let mut iv = [0; 16];

        for i in 0..16 {
            sym_key[i] = full_key[i];
            iv[i] = full_key[i + 16];
        }
        aes128_encrypt_slice(buffer_slice, iv, sym_key)
    }
}

mod test {
    use crate::encrypted_logs::body::EncryptedLogBody;
    use dep::protocol_types::{
        address::AztecAddress, traits::Empty, constants::GENERATOR_INDEX__NOTE_NULLIFIER,
        grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint
    };

    use crate::{
        note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
        oracle::{unsafe_rand::unsafe_rand, nullifier_key::get_app_nullifier_secret_key},
        context::PrivateContext, hash::poseidon2_hash
    };

    struct AddressNote {
        address: AztecAddress,
        owner: AztecAddress,
        randomness: Field,
        header: NoteHeader,
    }

    global ADDRESS_NOTE_LEN: Field = 3;

    impl NoteInterface<ADDRESS_NOTE_LEN> for AddressNote {
        fn compute_note_content_hash(self) -> Field {1}

        fn get_note_type_id() -> Field {2}

        fn get_header(self) -> NoteHeader { self.header}

        fn set_header(&mut self, header: NoteHeader) {self.header = header; }

        fn compute_nullifier(self, context: &mut PrivateContext) -> Field {1}

        fn compute_nullifier_without_context(self) -> Field {1}

        fn broadcast(self, context: &mut PrivateContext, slot: Field) {}

        fn serialize_content(self) -> [Field; ADDRESS_NOTE_LEN] { [self.address.to_field(), self.owner.to_field(), self.randomness]}

        fn deserialize_content(fields: [Field; ADDRESS_NOTE_LEN]) -> Self {
            AddressNote { address: AztecAddress::from_field(fields[0]), owner: AztecAddress::from_field(fields[1]), randomness: fields[2], header: NoteHeader::empty() }
        }
    }

    impl AddressNote {
        pub fn new(address: AztecAddress, owner: AztecAddress, randomness: Field) -> Self {
            AddressNote { address, owner, randomness, header: NoteHeader::empty() }
        }
    }

    #[test]
    fn test_encrypted_log_body() {
        let note = AddressNote::new(
            AztecAddress::from_field(0x1),
            AztecAddress::from_field(0x2),
            3
        );

        let note_type_id = 1;
        let storage_slot = 2;
        let body = EncryptedLogBody::new(storage_slot, note_type_id, note);

        let secret = GrumpkinPrivateKey::new(
            0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06,
            0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd
        );
        let point = GrumpkinPoint::new(
            0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e
        );

        let ciphertext = body.compute_ciphertext(secret, point);

        let expected_body_ciphertext = [
            131, 119, 105, 129, 244, 32, 151, 205, 12, 99, 93, 62, 10, 180, 72, 21, 47, 232, 95, 17, 240, 230, 80, 129, 174, 158, 23, 76, 114, 185, 43, 18, 254, 148, 147, 230, 66, 216, 167, 62, 180, 213, 238, 33, 108, 29, 84, 139, 99, 206, 212, 253, 92, 116, 137, 31, 0, 104, 45, 91, 250, 109, 141, 114, 189, 53, 35, 60, 108, 156, 170, 206, 150, 114, 150, 187, 198, 13, 62, 153, 133, 13, 169, 167, 242, 221, 40, 168, 186, 203, 104, 82, 47, 238, 142, 179, 90, 37, 9, 70, 245, 176, 122, 247, 42, 87, 75, 7, 20, 89, 166, 123, 14, 26, 230, 156, 49, 94, 0, 94, 72, 58, 171, 239, 115, 174, 155, 7, 151, 17, 60, 206, 193, 134, 70, 87, 215, 88, 21, 194, 63, 26, 106, 105, 124, 213, 252, 152, 192, 71, 115, 13, 181, 5, 169, 15, 170, 196, 174, 228, 170, 192, 91, 76, 110, 220, 89, 47, 248, 144, 189, 251, 167, 149, 248, 226
        ];

        for i in 0..expected_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_body_ciphertext[i]);
        }
        assert_eq(expected_body_ciphertext.len(), ciphertext.len());
    }
}
