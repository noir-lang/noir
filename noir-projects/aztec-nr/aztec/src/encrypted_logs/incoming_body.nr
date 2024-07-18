use crate::note::note_interface::NoteInterface;
use crate::event::event_interface::EventInterface;
use dep::protocol_types::{scalar::Scalar, point::Point};

use std::aes128::aes128_encrypt;
use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogIncomingBody<let M: u32> {
    plaintext: [u8; M]
}

impl<let M: u32> EncryptedLogIncomingBody<M> {
    pub fn from_note<T, let N: u32>(note: T, storage_slot: Field) -> Self where T: NoteInterface<N, M> {
        let mut plaintext = note.to_be_bytes(storage_slot);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn from_event<T, let MR: u32>(
        event: T,
        randomness: Field
    ) -> Self where T: EventInterface<M, MR> {
        let mut plaintext = event.private_to_be_bytes(randomness);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn compute_ciphertext(self, eph_sk: Scalar, ivpk: Point) -> [u8] {
        let full_key = point_to_symmetric_key(eph_sk, ivpk);
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
        scalar::Scalar, point::Point, traits::Serialize, abis::event_selector::EventSelector
    };

    use crate::{
        note::{note_header::NoteHeader, note_interface::NoteInterface},
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
        fn compute_note_content_hash(_self: Self) -> Field {1}

        fn get_note_type_id() -> Field {
            1
        }

        fn get_header(self) -> NoteHeader { self.header}

        fn set_header(&mut self, header: NoteHeader) {self.header = header; }

        fn compute_note_hash_and_nullifier(_self: Self, _context: &mut PrivateContext) -> (Field, Field) {
            (1, 1)
        }

        fn compute_note_hash_and_nullifier_without_context(_self: Self) -> (Field, Field) {(1,1)}

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

        let eph_sk = Scalar {
            lo: 0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd,
            hi: 0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06
        };
        let ivpk = Point {
            x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
            is_infinite: false
        };

        let body = EncryptedLogIncomingBody::from_note(note, storage_slot);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk);

        let expected_note_body_ciphertext = [
            166, 212, 106, 246, 139, 59, 228, 9, 133, 152, 127, 172, 141, 166, 237, 199, 195, 85, 255, 81, 66, 72, 192, 192, 96, 10, 54, 139, 136, 153, 252, 114, 248, 128, 253, 66, 249, 16, 71, 45, 2, 213, 250, 193, 241, 75, 90, 70, 39, 26, 104, 139, 20, 45, 1, 1, 166, 72, 133, 55, 247, 142, 150, 215, 217, 224, 84, 23, 245, 71, 207, 166, 136, 34, 221, 76, 90, 166, 44, 217, 246, 98, 157, 34, 198, 164, 99, 117, 15, 185, 145, 231, 189, 140, 201, 241, 135, 94, 71, 131, 156, 86, 144, 131, 248, 242, 83, 101, 18, 189, 1, 94, 25, 238, 76, 106, 85, 205, 4, 70, 21, 9, 64, 63, 27, 164, 73, 181, 75, 199, 86, 255, 105, 239, 216, 34, 217, 184, 154, 76, 67, 1, 210, 251, 23, 185, 114, 146, 195, 28, 76, 219, 150, 175, 37, 76, 144, 227, 99, 243, 123, 161, 66, 171, 148, 181, 162, 2, 196, 53, 207, 154, 114, 166, 155, 166
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
    global TEST_EVENT_BYTES_LEN_WITHOUT_RANDOMNESS = 32 * 3 + 32;

    impl EventInterface<TEST_EVENT_BYTES_LEN, TEST_EVENT_BYTES_LEN_WITHOUT_RANDOMNESS> for TestEvent {
        fn get_event_type_id() -> EventSelector {
            EventSelector::from_signature("TestEvent(Field,Field,Field)")
        }

        fn private_to_be_bytes(self, randomness: Field) -> [u8; TEST_EVENT_BYTES_LEN] {
            let mut buffer: [u8; TEST_EVENT_BYTES_LEN] = [0; TEST_EVENT_BYTES_LEN];

            let randomness_bytes = randomness.to_be_bytes(32);
            let event_type_id_bytes = TestEvent::get_event_type_id().to_field().to_be_bytes(32);

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

        fn to_be_bytes(self) -> [u8; TEST_EVENT_BYTES_LEN_WITHOUT_RANDOMNESS] {
             let mut buffer: [u8; TEST_EVENT_BYTES_LEN_WITHOUT_RANDOMNESS] = [0; TEST_EVENT_BYTES_LEN_WITHOUT_RANDOMNESS];

             let event_type_id_bytes = TestEvent::get_event_type_id().to_field().to_be_bytes(32);

             for i in 0..32 {
                 buffer[i] = event_type_id_bytes[i];
            }

             let serialized_event = self.serialize();

             for i in 0..serialized_event.len() {
                 let bytes = serialized_event[i].to_be_bytes(32);
                 for j in 0..32 {
                     buffer[32 + i * 32 + j] = bytes[j];
                }
            }

             buffer
        }

        fn emit<Env>(self, _emit: fn[Env](Self) -> ()) {
            _emit(self);
        }
    }

    #[test]
    fn test_encrypted_log_event_incoming_body() {
        let test_event = TestEvent { value0: 1, value1: 2, value2: 3 };

        let eph_sk = Scalar {
            lo: 0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd,
            hi: 0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06
        };

        let ivpk = Point {
            x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
            is_infinite: false
        };

        let randomness = 2;

        let body = EncryptedLogIncomingBody::from_event(test_event, randomness);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk);

        let expected_event_body_ciphertext = [
            166, 212, 106, 246, 139, 59, 228, 9, 133, 152, 127, 172, 141, 166, 237, 199, 195, 85, 255, 81, 66, 72, 192, 192, 96, 10, 54, 139, 136, 153, 252, 114, 248, 128, 253, 66, 249, 16, 71, 45, 2, 213, 250, 193, 241, 75, 90, 70, 19, 153, 62, 117, 71, 55, 48, 114, 160, 232, 97, 118, 93, 53, 145, 92, 0, 225, 51, 81, 156, 69, 72, 224, 10, 89, 32, 121, 167, 197, 84, 245, 188, 235, 143, 202, 179, 197, 164, 121, 11, 105, 116, 239, 46, 222, 50, 138, 112, 237, 97, 8, 176, 199, 1, 151, 89, 218, 60, 45, 91, 85, 16, 38, 195, 127, 157, 182, 0, 10, 232, 184, 148, 76, 244, 63, 40, 222, 219, 139, 236, 169, 213, 17, 32, 210, 50, 6, 5, 83, 80, 1, 111, 246, 197, 83, 166, 71, 31, 246, 234, 75, 12, 151, 227, 247, 143, 229, 95, 219, 159, 75, 174, 232, 64, 7, 102, 76, 207, 45, 143, 208, 101, 113, 175, 37, 83, 166
        ];

        assert_eq(expected_event_body_ciphertext.len(), ciphertext.len());

        for i in 0..expected_event_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_event_body_ciphertext[i]);
        }
    }
}
