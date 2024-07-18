use dep::protocol_types::{
    address::AztecAddress, scalar::Scalar, point::Point,
    constants::{GENERATOR_INDEX__IVSK_M, GENERATOR_INDEX__OVSK_M}, hash::poseidon2_hash
};
use std::embedded_curve_ops::fixed_base_scalar_mul as derive_public_key;
use std::field::bytes32_to_field;

use crate::oracle::unsafe_rand::unsafe_rand;
use crate::utils::point::point_to_bytes;

use crate::event::event_interface::EventInterface;
use crate::note::note_interface::NoteInterface;

use crate::encrypted_logs::{
    header::EncryptedLogHeader, incoming_body::EncryptedLogIncomingBody,
    outgoing_body::EncryptedLogOutgoingBody
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
    // @todo Need to draw randomness from the full domain of Fq not only Fr
    let eph_sk: Scalar = fr_to_fq(unsafe_rand());
    let eph_pk = derive_public_key(eph_sk);

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
    // @todo Need to draw randomness from the full domain of Fq not only Fr
    let eph_sk: Scalar = fr_to_fq(unsafe_rand());
    let eph_pk = derive_public_key(eph_sk);

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

/// Converts a base field elememt to scalar field element.
/// This is fine because modulus of the base field is smaller than the modulus of the scalar field.
fn fr_to_fq(r: Field) -> Scalar {
    let r_bytes = r.to_be_bytes(32);

    let mut high_bytes = [0; 32];
    let mut low_bytes = [0; 32];

    for i in 0..16 {
        high_bytes[16 + i] = r_bytes[i];
        low_bytes[16 + i] = r_bytes[i + 16];
    }

    let lo = bytes32_to_field(low_bytes);
    let hi = bytes32_to_field(high_bytes);

    Scalar { lo, hi }
}

mod test {
    use crate::{encrypted_logs::payload::compute_encrypted_note_log, test::mocks::mock_note::MockNoteBuilder};
    use dep::protocol_types::{address::AztecAddress, point::Point};
    use std::test::OracleMock;

    #[test]
    fn test_compute_encrypted_note_log() {
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

        let expected_encrypted_note_log = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 141, 70, 12, 14, 67, 77, 132, 110, 193, 234, 40, 110, 64, 144, 235, 86, 55, 111, 242, 123, 221, 193, 170, 202, 225, 216, 86, 84, 159, 112, 31, 167, 159, 53, 114, 117, 237, 57, 131, 19, 111, 150, 50, 83, 173, 155, 234, 225, 71, 187, 141, 79, 245, 43, 111, 83, 219, 149, 124, 68, 12, 244, 253, 216, 0, 62, 108, 232, 118, 80, 87, 140, 215, 185, 111, 48, 128, 236, 110, 92, 46, 205, 7, 226, 131, 66, 205, 0, 103, 83, 217, 90, 60, 138, 6, 172, 246, 129, 92, 172, 69, 73, 77, 65, 147, 18, 231, 20, 35, 217, 180, 253, 72, 242, 32, 57, 45, 11, 2, 235, 24, 96, 244, 224, 33, 61, 151, 225, 136, 173, 178, 40, 2, 125, 229, 20, 220, 82, 28, 191, 147, 133, 137, 1, 45, 243, 229, 140, 115, 165, 150, 154, 96, 22, 120, 223, 237, 213, 182, 252, 192, 8, 132, 43, 21, 56, 243, 116, 144, 182, 75, 16, 30, 222, 222, 60, 205, 147, 214, 53, 41, 62, 53, 16, 147, 117, 72, 169, 220, 125, 208, 210, 45, 65, 233, 40, 87, 88, 140, 237, 200, 161, 9, 86, 82, 128, 191, 51, 4, 195, 243, 100, 102, 240, 54, 129, 176, 116, 139, 73, 27, 98, 222, 1, 243, 199, 72, 238, 213, 66, 91, 159, 183, 143, 36, 103, 94, 5, 62, 50, 13, 217, 161, 79, 30, 231, 41, 228, 109, 139, 243, 119, 166, 54, 37, 250, 193, 6, 67, 29, 148, 185, 153, 58, 64, 210, 164, 219, 165, 80, 35, 75, 109, 177, 14, 168, 136, 105, 21, 235, 62, 159, 71, 61, 245, 193, 234, 169, 100, 165, 8, 222, 157, 239, 41, 221, 223, 67, 80, 61, 252, 54, 27, 100, 1, 104, 2, 121, 62, 41, 23, 132, 15, 124, 120, 21, 198, 113, 151, 172, 42, 161, 64, 240, 166, 205, 80, 169, 58, 191, 111, 130, 55, 58, 141, 26, 97, 118, 114, 216, 69, 207, 212, 227, 250, 199, 21, 72, 144, 85, 43, 76, 213, 28, 132, 134, 16, 221, 105, 112, 82, 238, 114, 61, 36, 144, 179, 178, 68, 198, 162, 212, 85, 100, 116, 186, 131, 232, 33, 229, 101, 251, 5, 251
        ];
        for i in 0..expected_encrypted_note_log.len() {
            assert_eq(log[i], expected_encrypted_note_log[i]);
        }
        assert_eq(expected_encrypted_note_log.len(), log.len());
    }
}
