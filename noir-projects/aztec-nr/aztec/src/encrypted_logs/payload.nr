use dep::protocol_types::{address::AztecAddress, scalar::Scalar, point::Point};
use std::{
    embedded_curve_ops::fixed_base_scalar_mul as derive_public_key,
    hash::from_field_unsafe as fr_to_fq_unsafe, field::bn254::decompose
};

use crate::{
    event::event_interface::EventInterface, oracle::unsafe_rand::unsafe_rand,
    utils::point::point_to_bytes, note::note_interface::NoteInterface,
    encrypted_logs::{
    header::EncryptedLogHeader, incoming_body::EncryptedLogIncomingBody,
    outgoing_body::EncryptedLogOutgoingBody
}
};

pub fn compute_encrypted_event_log<Event, let NB: u32, let MB: u32, let OB: u32>(
    contract_address: AztecAddress,
    randomness: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress,
    event: Event
) -> [u8; OB] where Event: EventInterface<NB, MB> {
    let (eph_sk, eph_pk) = generate_ephemeral_key_pair();

    let header = EncryptedLogHeader::new(contract_address);

    let incoming_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ivpk);
    let outgoing_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ovpk);
    let incoming_body_ciphertext = EncryptedLogIncomingBody::from_event(event, randomness).compute_ciphertext(eph_sk, ivpk);
    let outgoing_body_ciphertext: [u8; 144] = EncryptedLogOutgoingBody::new(eph_sk, recipient, ivpk).compute_ciphertext(fr_to_fq(ovsk_app), eph_pk);

    let mut encrypted_bytes: [u8; OB] = [0; OB];
    // @todo We ignore the tags for now 

    let eph_pk_bytes = point_to_bytes(eph_pk);
    for i in 0..32 {
        encrypted_bytes[64 + i] = eph_pk_bytes[i];
    }
    for i in 0..48 {
        encrypted_bytes[96 + i] = incoming_header_ciphertext[i];
        encrypted_bytes[144 + i] = outgoing_header_ciphertext[i];
    }
    for i in 0..144 {
        encrypted_bytes[192 + i] = outgoing_body_ciphertext[i];
    }
    // Then we fill in the rest as the incoming body ciphertext
    let size = OB - 336;
    assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
    for i in 0..size {
        encrypted_bytes[336 + i] = incoming_body_ciphertext[i];
    }

    // Current unoptimized size of the encrypted log
    // incoming_tag (32 bytes)
    // outgoing_tag (32 bytes)
    // eph_pk (32 bytes)
    // incoming_header (48 bytes)
    // outgoing_header (48 bytes)
    // outgoing_body (144 bytes)
    // incoming_body_fixed (64 bytes)
    // incoming_body_variable (N * 32 bytes + 16 bytes padding)
    encrypted_bytes
}

pub fn compute_encrypted_note_log<Note, let N: u32, let NB: u32, let M: u32>(
    contract_address: AztecAddress,
    storage_slot: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress,
    note: Note
) -> [u8; M] where Note: NoteInterface<N, NB> {
    let (eph_sk, eph_pk) = generate_ephemeral_key_pair();

    let header = EncryptedLogHeader::new(contract_address);

    let incoming_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ivpk);
    let outgoing_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ovpk);
    let incoming_body_ciphertext = EncryptedLogIncomingBody::from_note(note, storage_slot).compute_ciphertext(eph_sk, ivpk);
    let outgoing_body_ciphertext: [u8; 144] = EncryptedLogOutgoingBody::new(eph_sk, recipient, ivpk).compute_ciphertext(fr_to_fq(ovsk_app), eph_pk);

    let mut encrypted_bytes: [u8; M] = [0; M];
    // @todo We ignore the tags for now 

    let eph_pk_bytes = point_to_bytes(eph_pk);
    for i in 0..32 {
        encrypted_bytes[64 + i] = eph_pk_bytes[i];
    }
    for i in 0..48 {
        encrypted_bytes[96 + i] = incoming_header_ciphertext[i];
        encrypted_bytes[144 + i] = outgoing_header_ciphertext[i];
    }
    for i in 0..144 {
        encrypted_bytes[192 + i] = outgoing_body_ciphertext[i];
    }
    // Then we fill in the rest as the incoming body ciphertext
    let size = M - 336;
    assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
    for i in 0..size {
        encrypted_bytes[336 + i] = incoming_body_ciphertext[i];
    }

    // Current unoptimized size of the encrypted log
    // incoming_tag (32 bytes)
    // outgoing_tag (32 bytes)
    // eph_pk (32 bytes)
    // incoming_header (48 bytes)
    // outgoing_header (48 bytes)
    // outgoing_body (144 bytes)
    // incoming_body_fixed (64 bytes)
    // incoming_body_variable (N * 32 bytes + 16 bytes padding)
    encrypted_bytes
}

/// Converts a base field element to scalar field element.
/// This is fine because modulus of the base field is smaller than the modulus of the scalar field.
fn fr_to_fq(r: Field) -> Scalar {
    let (lo, hi) = decompose(r);

    Scalar { lo, hi }
}

fn generate_ephemeral_key_pair() -> (Scalar, Point) {
    // @todo Need to draw randomness from the full domain of Fq not only Fr
    // We use the unsafe version of `fr_to_fq` because multi_scalar_mul (called by derive_public_key) will constrain
    // the scalars.
    let eph_sk = fr_to_fq_unsafe(unsafe_rand());
    let eph_pk = derive_public_key(eph_sk);

    (eph_sk, eph_pk)
}

mod test {
    use crate::{encrypted_logs::payload::compute_encrypted_note_log, test::mocks::mock_note::MockNoteBuilder};
    use dep::protocol_types::{address::AztecAddress, point::Point};
    use std::test::OracleMock;

    #[test]
    fn test_encrypted_note_log_matches_typescript() {
        // All the values in this test were copied over from `tagged_log.test.ts`
        let contract_address = AztecAddress::from_field(0x10f48cd9eff7ae5b209c557c70de2e657ee79166868676b787e9417e19260e04);
        let storage_slot = 0x0fe46be583b71f4ab5b70c2657ff1d05cccf1d292a9369628d1a194f944e6599;
        let ovsk_app = 0x1b99ba138fa7ef8a2f122a98dd80c8ee70d447218dd780f45e165ac17ca38a5e;
        let ovpk_m = Point {
            x: 0x1961448682803198631f299340e4206bb12809d4bebbf012b30f59af73ba1a15,
            y: 0x133674060c3925142aceb4f1dcd9f9137d0217d37ff8729ee5ceaa6e2790353d,
            is_infinite: false
        };
        let ivpk_m = Point {
            x: 0x260cd3904f6df16e974c29944fdc839e40fb5cf293f03df2eb370851d3a527bc,
            y: 0x0eef2964fe6640e84c82b5d2915892409b38e9e25d39f68dd79edb725c55387f,
            is_infinite: false
        };

        let note_value = 0x301640ceea758391b2e161c92c0513f129020f4125256afdae2646ce31099f5c;
        let note = MockNoteBuilder::new(note_value).contract_address(contract_address).storage_slot(storage_slot).build();

        let eph_sk = 0x1358d15019d4639393d62b97e1588c095957ce74a1c32d6ec7d62fe6705d9538;
        let _ = OracleMock::mock("getRandomField").returns(eph_sk);

        let recipient = AztecAddress::from_field(0x10ee41ee4b62703b16f61e03cb0d88c4b306a9eb4a6ceeb2aff13428541689a2);

        let log: [u8; 448] = compute_encrypted_note_log(
            contract_address,
            storage_slot,
            ovsk_app,
            ovpk_m,
            ivpk_m,
            recipient,
            note
        );

        // The following value was generated by `tagged_log.test.ts`
        // --> Run the test with AZTEC_GENERATE_TEST_DATA=1 flag to update test data.
        let encrypted_note_log_from_typescript = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 141, 70, 12, 14, 67, 77, 132, 110, 193, 234, 40, 110, 64, 144, 235, 86, 55, 111, 242, 123, 221, 193, 170, 202, 225, 216, 86, 84, 159, 112, 31, 167, 126, 79, 51, 186, 47, 71, 253, 172, 99, 112, 241, 59, 197, 241, 107, 186, 232, 87, 187, 230, 171, 62, 228, 234, 42, 51, 145, 146, 238, 242, 42, 71, 206, 13, 244, 66, 111, 195, 20, 203, 98, 148, 204, 242, 145, 183, 156, 29, 141, 54, 44, 220, 194, 35, 229, 16, 32, 204, 211, 49, 142, 112, 82, 202, 116, 241, 254, 146, 42, 217, 20, 189, 70, 228, 182, 171, 205, 104, 27, 99, 171, 28, 91, 244, 21, 30, 130, 240, 5, 72, 174, 124, 97, 197, 157, 248, 204, 203, 140, 171, 181, 152, 130, 169, 179, 41, 52, 173, 45, 43, 198, 1, 152, 72, 158, 249, 11, 41, 9, 160, 48, 78, 123, 132, 203, 140, 215, 13, 22, 201, 88, 255, 139, 154, 76, 20, 63, 134, 125, 108, 239, 208, 63, 59, 33, 117, 139, 225, 184, 0, 64, 153, 21, 131, 204, 111, 41, 84, 23, 144, 222, 245, 200, 12, 234, 11, 48, 10, 221, 20, 252, 38, 122, 40, 249, 66, 248, 197, 198, 209, 79, 20, 59, 66, 197, 215, 16, 18, 145, 228, 239, 124, 81, 67, 103, 49, 196, 58, 228, 195, 64, 199, 243, 184, 112, 173, 29, 196, 215, 77, 217, 85, 82, 149, 113, 76, 201, 93, 95, 148, 37, 95, 222, 233, 210, 150, 1, 182, 28, 132, 59, 148, 156, 129, 36, 230, 55, 199, 149, 36, 205, 103, 212, 60, 151, 141, 10, 151, 222, 151, 180, 43, 91, 148, 201, 110, 165, 10, 238, 32, 134, 235, 99, 216, 200, 182, 31, 22, 156, 18, 209, 222, 172, 239, 193, 212, 86, 99, 62, 70, 182, 45, 175, 241, 91, 202, 179, 225, 99, 1, 150, 232, 2, 252, 20, 83, 49, 132, 162, 93, 116, 212, 87, 71, 211, 58, 159, 163, 40, 253, 31, 3, 192, 48, 14, 201, 80, 24, 135, 154, 207, 58, 140, 128, 29, 101, 207, 189, 182, 191, 71, 210, 64, 172, 131, 83, 46, 232, 19, 216, 183, 108, 234, 17, 104, 60, 113, 231, 145, 195, 157, 24
        ];
        for i in 0..encrypted_note_log_from_typescript.len() {
            assert_eq(log[i], encrypted_note_log_from_typescript[i]);
        }
        assert_eq(encrypted_note_log_from_typescript.len(), log.len());
    }
}
