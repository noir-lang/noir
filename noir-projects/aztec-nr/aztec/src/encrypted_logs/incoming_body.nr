use crate::note::note_interface::NoteInterface;
use crate::event::event_interface::EventInterface;
use dep::protocol_types::{grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint};

use dep::std::aes128::aes128_encrypt;
use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogIncomingBody<M> {
    plaintext: [u8; M]
}

impl<M> EncryptedLogIncomingBody<M> {
    pub fn from_note<T, N>(note: T, storage_slot: Field) -> Self where T: NoteInterface<N, M> {
        let mut plaintext = note.to_be_bytes(storage_slot);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn from_event<T>(event: T, randomness: Field) -> Self where T: EventInterface<M> {
        let mut plaintext = event.to_be_bytes(randomness);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn compute_ciphertext(self, eph_sk: GrumpkinPrivateKey, ivpk_app: GrumpkinPoint) -> [u8] {
        let full_key = point_to_symmetric_key(eph_sk, ivpk_app);
        let mut sym_key = [0; 16];
        let mut iv = [0; 16];

        for i in 0..16 {
            sym_key[i] = full_key[i];
            iv[i] = full_key[i + 16];
        }
        aes128_encrypt(self.plaintext, iv, sym_key)
    }
}

mod test {
    use crate::encrypted_logs::incoming_body::EncryptedLogIncomingBody;
    use dep::protocol_types::{
        address::AztecAddress, traits::Empty, constants::GENERATOR_INDEX__NOTE_NULLIFIER,
        grumpkin_private_key::GrumpkinPrivateKey, grumpkin_point::GrumpkinPoint, traits::Serialize,
        abis::function_selector::FunctionSelector
    };

    use crate::{
        note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_consumption},
        event::event_interface::EventInterface, oracle::unsafe_rand::unsafe_rand,
        context::PrivateContext
    };

    struct AddressNote {
        address: AztecAddress,
        owner: AztecAddress,
        randomness: Field,
        header: NoteHeader,
    }

    global ADDRESS_NOTE_LEN: Field = 3;
    global ADDRESS_NOTE_BYTES_LEN = 32 * 3 + 64;

    impl NoteInterface<ADDRESS_NOTE_LEN, ADDRESS_NOTE_BYTES_LEN> for AddressNote {
        fn compute_note_content_hash(self) -> Field {1}

        fn get_note_type_id() -> Field {1}

        fn get_header(self) -> NoteHeader { self.header}

        fn set_header(&mut self, header: NoteHeader) {self.header = header; }

        fn compute_nullifier(self, context: &mut PrivateContext) -> Field {1}

        fn compute_nullifier_without_context(self) -> Field {1}

        fn broadcast(self, context: &mut PrivateContext, slot: Field, ovpk_m: GrumpkinPoint, ivpk_m: GrumpkinPoint) {}

        fn serialize_content(self) -> [Field; ADDRESS_NOTE_LEN] { [self.address.to_field(), self.owner.to_field(), self.randomness]}

        fn deserialize_content(fields: [Field; ADDRESS_NOTE_LEN]) -> Self {
            AddressNote { address: AztecAddress::from_field(fields[0]), owner: AztecAddress::from_field(fields[1]), randomness: fields[2], header: NoteHeader::empty() }
        }

        fn to_be_bytes(self, storage_slot: Field) -> [u8; ADDRESS_NOTE_BYTES_LEN] {
            let serialized_note = self.serialize_content();

            let mut buffer: [u8; ADDRESS_NOTE_BYTES_LEN] = [0; ADDRESS_NOTE_BYTES_LEN];

            let storage_slot_bytes = storage_slot.to_be_bytes(32);
            let note_type_id_bytes = AddressNote::get_note_type_id().to_be_bytes(32);

            for i in 0..32 {
                buffer[i] = storage_slot_bytes[i];
                buffer[32 + i] = note_type_id_bytes[i];
            }

            for i in 0..serialized_note.len() {
                let bytes = serialized_note[i].to_be_bytes(32);
                for j in 0..32 {
                    buffer[64 + i * 32 + j] = bytes[j];
                }
            }
            buffer
        }
    }

    impl AddressNote {
        pub fn new(address: AztecAddress, owner: AztecAddress, randomness: Field) -> Self {
            AddressNote { address, owner, randomness, header: NoteHeader::empty() }
        }
    }

    #[test]
    fn test_encrypted_note_log_incoming_body() {
        let note = AddressNote::new(
            AztecAddress::from_field(0x1),
            AztecAddress::from_field(0x2),
            3
        );

        let storage_slot = 2;

        let eph_sk = GrumpkinPrivateKey::new(
            0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06,
            0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd
        );
        let ivpk_app = GrumpkinPoint::new(
            0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e
        );

        let body = EncryptedLogIncomingBody::from_note(note, storage_slot);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk_app);

        let expected_note_body_ciphertext = [
            131, 119, 105, 129, 244, 32, 151, 205, 12, 99, 93, 62, 10, 180, 72, 21, 47, 232, 95, 17, 240, 230, 80, 129, 174, 158, 23, 76, 114, 185, 43, 18, 254, 148, 147, 230, 66, 216, 167, 62, 180, 213, 238, 33, 108, 29, 84, 139, 99, 206, 212, 253, 92, 116, 137, 31, 0, 104, 45, 91, 250, 109, 141, 114, 189, 53, 35, 60, 108, 156, 170, 206, 150, 114, 150, 187, 198, 13, 62, 153, 133, 13, 169, 167, 242, 221, 40, 168, 186, 203, 104, 82, 47, 238, 142, 179, 90, 37, 9, 70, 245, 176, 122, 247, 42, 87, 75, 7, 20, 89, 166, 123, 14, 26, 230, 156, 49, 94, 0, 94, 72, 58, 171, 239, 115, 174, 155, 7, 151, 17, 60, 206, 193, 134, 70, 87, 215, 88, 21, 194, 63, 26, 106, 105, 124, 213, 252, 152, 192, 71, 115, 13, 181, 5, 169, 15, 170, 196, 174, 228, 170, 192, 91, 76, 110, 220, 89, 47, 248, 144, 189, 251, 167, 149, 248, 226
        ];

        assert_eq(expected_note_body_ciphertext.len(), ciphertext.len());

        for i in 0..expected_note_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_note_body_ciphertext[i]);
        }
    }

    struct TestEvent {
        value0: Field,
        value1: Field,
        value2: Field,
    }

    impl Serialize<3> for TestEvent {
        fn serialize(self) -> [Field; 3] {
            [self.value0, self.value1, self.value2]
        }
    }

    global TEST_EVENT_LEN: Field = 3;
    global TEST_EVENT_BYTES_LEN = 32 * 3 + 64;

    impl EventInterface<TEST_EVENT_BYTES_LEN> for TestEvent {
        fn _selector(self) -> FunctionSelector {
            FunctionSelector::from_signature("TestEvent(Field,Field,Field)")
        }

        fn to_be_bytes(self, randomness: Field) -> [u8; TEST_EVENT_BYTES_LEN] {
            let mut buffer: [u8; TEST_EVENT_BYTES_LEN] = [0; TEST_EVENT_BYTES_LEN];

            let randomness_bytes = randomness.to_be_bytes(32);
            let event_type_id_bytes = self._selector().to_field().to_be_bytes(32);

            for i in 0..32 {
                buffer[i] = randomness_bytes[i];
                buffer[32 + i] = event_type_id_bytes[i];
            }

            let serialized_event = self.serialize();

            for i in 0..serialized_event.len() {
                let bytes = serialized_event[i].to_be_bytes(32);
                for j in 0..32 {
                    buffer[64 + i * 32 + j] = bytes[j];
                }
            }

            buffer
        }
    }

    #[test]
    fn test_encrypted_log_event_incoming_body() {
        let test_event = TestEvent { value0: 1, value1: 2, value2: 3 };

        let eph_sk = GrumpkinPrivateKey::new(
            0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06,
            0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd
        );

        let ivpk_app = GrumpkinPoint::new(
            0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e
        );

        let randomness = 2;

        let body = EncryptedLogIncomingBody::from_event(test_event, randomness);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk_app);

        let expected_event_body_ciphertext = [
            131, 119, 105, 129, 244, 32, 151, 205, 12, 99, 93, 62, 10, 180, 72, 21, 47, 232, 95, 17, 240, 230, 80, 129, 174, 158, 23, 76, 114, 185, 43, 18, 254, 148, 147, 230, 66, 216, 167, 62, 180, 213, 238, 33, 108, 29, 84, 139, 157, 165, 187, 138, 35, 3, 236, 75, 197, 105, 102, 247, 224, 253, 13, 217, 145, 62, 96, 167, 93, 23, 18, 198, 187, 91, 8, 3, 197, 195, 127, 9, 218, 111, 125, 97, 141, 129, 142, 1, 230, 108, 35, 211, 170, 170, 170, 249, 249, 104, 68, 191, 245, 207, 182, 245, 248, 82, 175, 83, 155, 138, 208, 65, 31, 129, 251, 242, 219, 76, 17, 61, 178, 187, 108, 114, 177, 215, 175, 189, 166, 221, 94, 9, 22, 57, 151, 204, 57, 220, 129, 243, 217, 18, 101, 128, 229, 40, 254, 175, 2, 21, 31, 198, 18, 152, 169, 32, 113, 92, 37, 65, 169, 119, 95, 149, 239, 8, 23, 182, 22, 209, 207, 120, 133, 90, 252, 106
        ];

        assert_eq(expected_event_body_ciphertext.len(), ciphertext.len());

        for i in 0..expected_event_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_event_body_ciphertext[i]);
        }
    }
}
