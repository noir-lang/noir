use crate::note::note_interface::NoteInterface;
use crate::event::event_interface::EventInterface;
use dep::protocol_types::{scalar::Scalar, point::Point};

use std::aes128::aes128_encrypt;
use crate::keys::point_to_symmetric_key::point_to_symmetric_key;

struct EncryptedLogIncomingBody<M> {
    plaintext: [u8; M]
}

impl<M> EncryptedLogIncomingBody<M> {
    pub fn from_note<T, N>(note: T, storage_slot: Field) -> Self where T: NoteInterface<N, M> {
        let mut plaintext = note.to_be_bytes(storage_slot);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn from_event<T, MR>(event: T, randomness: Field) -> Self where T: EventInterface<M, MR> {
        let mut plaintext = event.private_to_be_bytes(randomness);
        EncryptedLogIncomingBody { plaintext }
    }

    pub fn compute_ciphertext(self, eph_sk: Scalar, ivpk_app: Point) -> [u8] {
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
        fn compute_note_content_hash(self) -> Field {1}

        fn get_note_type_id() -> Field {
            1
        }

        fn get_header(self) -> NoteHeader { self.header}

        fn set_header(&mut self, header: NoteHeader) {self.header = header; }

        fn compute_note_hash_and_nullifier(self, context: &mut PrivateContext) -> (Field, Field) {
            (1, 1)
        }

        fn compute_note_hash_and_nullifier_without_context(self) -> (Field, Field) {(1,1)}

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
        let ivpk_app = Point {
            x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
            is_infinite: false
        };

        let body = EncryptedLogIncomingBody::from_note(note, storage_slot);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk_app);

        let expected_note_body_ciphertext = [
            228, 9, 65, 81, 62, 59, 249, 207, 90, 196, 206, 72, 39, 199, 82, 196, 63, 127, 188, 251, 150, 188, 238, 205, 3, 86, 102, 164, 175, 12, 137, 158, 163, 111, 205, 10, 229, 230, 46, 202, 110, 107, 156, 180, 67, 192, 161, 201, 48, 153, 169, 1, 25, 182, 93, 39, 39, 207, 251, 218, 234, 147, 156, 13, 110, 180, 190, 199, 41, 6, 211, 203, 176, 110, 165, 186, 110, 127, 199, 22, 201, 149, 92, 249, 219, 68, 145, 68, 179, 29, 233, 34, 98, 123, 197, 234, 169, 53, 44, 14, 81, 60, 92, 27, 250, 134, 49, 248, 57, 119, 236, 118, 158, 104, 82, 243, 98, 164, 60, 72, 74, 27, 177, 194, 221, 225, 193, 150, 67, 235, 205, 106, 150, 24, 126, 186, 220, 178, 199, 189, 113, 54, 181, 55, 46, 15, 236, 236, 9, 159, 5, 172, 237, 154, 110, 50, 241, 64, 92, 13, 37, 53, 20, 140, 42, 146, 229, 63, 97, 25, 159, 63, 235, 104, 68, 100
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

        let ivpk_app = Point {
            x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
            y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
            is_infinite: false
        };

        let randomness = 2;

        let body = EncryptedLogIncomingBody::from_event(test_event, randomness);

        let ciphertext = body.compute_ciphertext(eph_sk, ivpk_app);

        let expected_event_body_ciphertext = [
            228, 9, 65, 81, 62, 59, 249, 207, 90, 196, 206, 72, 39, 199, 82, 196, 63, 127, 188, 251, 150, 188, 238, 205, 3, 86, 102, 164, 175, 12, 137, 158, 163, 111, 205, 10, 229, 230, 46, 202, 110, 107, 156, 180, 67, 192, 161, 201, 66, 122, 29, 35, 42, 33, 153, 216, 199, 208, 103, 207, 126, 153, 189, 136, 19, 220, 238, 15, 169, 29, 255, 11, 123, 107, 70, 192, 53, 40, 36, 93, 187, 32, 123, 136, 104, 23, 229, 245, 152, 90, 84, 2, 136, 112, 42, 27, 82, 214, 104, 14, 250, 48, 199, 245, 88, 22, 200, 77, 38, 51, 127, 56, 138, 255, 16, 46, 179, 129, 215, 185, 185, 116, 148, 16, 133, 62, 56, 180, 10, 132, 109, 77, 206, 199, 21, 167, 7, 163, 171, 158, 244, 23, 18, 121, 108, 42, 107, 7, 48, 84, 212, 104, 39, 16, 109, 7, 108, 129, 60, 80, 112, 241, 223, 140, 186, 158, 38, 74, 230, 213, 159, 175, 142, 228, 128, 160
        ];

        assert_eq(expected_event_body_ciphertext.len(), ciphertext.len());

        for i in 0..expected_event_body_ciphertext.len() {
            assert_eq(ciphertext[i], expected_event_body_ciphertext[i]);
        }
    }
}
