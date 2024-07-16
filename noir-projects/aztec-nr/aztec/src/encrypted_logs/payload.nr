use dep::protocol_types::{
    address::AztecAddress, scalar::Scalar, point::{Point, pub_key_to_bytes},
    constants::{GENERATOR_INDEX__IVSK_M, GENERATOR_INDEX__OVSK_M}, hash::poseidon2_hash
};
use std::embedded_curve_ops::fixed_base_scalar_mul as derive_public_key;
use std::field::bytes32_to_field;

use crate::oracle::unsafe_rand::unsafe_rand;

use crate::event::event_interface::EventInterface;
use crate::note::note_interface::NoteInterface;

use crate::encrypted_logs::{
    header::EncryptedLogHeader, incoming_body::EncryptedLogIncomingBody,
    outgoing_body::EncryptedLogOutgoingBody
};

pub fn compute_encrypted_event_log<Event, NB, MB, OB>(
    contract_address: AztecAddress,
    randomness: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    event: Event
) -> [u8; OB] where Event: EventInterface<NB, MB> {
    // @todo Need to draw randomness from the full domain of Fq not only Fr
    let eph_sk: Scalar = fr_to_fq(unsafe_rand());
    let eph_pk = derive_public_key(eph_sk);

    // TODO: (#7177) This value needs to be populated!
    let recipient = AztecAddress::from_field(0);

    let ivpk_app = compute_ivpk_app(ivpk, contract_address);

    let header = EncryptedLogHeader::new(contract_address);

    let incoming_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ivpk);
    let outgoing_Header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ovpk);
    let incoming_body_ciphertext = EncryptedLogIncomingBody::from_event(event, randomness).compute_ciphertext(eph_sk, ivpk_app);
    let outgoing_body_ciphertext: [u8; 176] = EncryptedLogOutgoingBody::new(eph_sk, recipient, ivpk_app).compute_ciphertext(fr_to_fq(ovsk_app), eph_pk);

    let mut encrypted_bytes: [u8; OB] = [0; OB];
    // @todo We ignore the tags for now 

    let eph_pk_bytes = pub_key_to_bytes(eph_pk);
    for i in 0..64 {
        encrypted_bytes[64 + i] = eph_pk_bytes[i];
    }
    for i in 0..48 {
        encrypted_bytes[128 + i] = incoming_header_ciphertext[i];
        encrypted_bytes[176 + i] = outgoing_Header_ciphertext[i];
    }
    for i in 0..176 {
        encrypted_bytes[224 + i] = outgoing_body_ciphertext[i];
    }
    // Then we fill in the rest as the incoming body ciphertext
    let size = OB - 400;
    assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
    for i in 0..size {
        encrypted_bytes[400 + i] = incoming_body_ciphertext[i];
    }

    // Current unoptimized size of the encrypted log
    // incoming_tag (32 bytes)
    // outgoing_tag (32 bytes)
    // eph_pk (64 bytes)
    // incoming_header (48 bytes)
    // outgoing_header (48 bytes)
    // outgoing_body (176 bytes)
    // incoming_body_fixed (64 bytes)
    // incoming_body_variable (N * 32 bytes + 16 bytes padding)
    encrypted_bytes
}

pub fn compute_encrypted_note_log<Note, N, NB, M>(
    contract_address: AztecAddress,
    storage_slot: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    note: Note
) -> [u8; M] where Note: NoteInterface<N, NB> {
    // @todo Need to draw randomness from the full domain of Fq not only Fr
    let eph_sk: Scalar = fr_to_fq(unsafe_rand());
    let eph_pk = derive_public_key(eph_sk);

    // TODO: (#7177) This value needs to be populated!
    let recipient = AztecAddress::from_field(0);

    let ivpk_app = compute_ivpk_app(ivpk, contract_address);

    let header = EncryptedLogHeader::new(contract_address);

    let incoming_header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ivpk);
    let outgoing_Header_ciphertext: [u8; 48] = header.compute_ciphertext(eph_sk, ovpk);
    let incoming_body_ciphertext = EncryptedLogIncomingBody::from_note(note, storage_slot).compute_ciphertext(eph_sk, ivpk_app);
    let outgoing_body_ciphertext: [u8; 176] = EncryptedLogOutgoingBody::new(eph_sk, recipient, ivpk_app).compute_ciphertext(fr_to_fq(ovsk_app), eph_pk);

    let mut encrypted_bytes: [u8; M] = [0; M];
    // @todo We ignore the tags for now 

    let eph_pk_bytes = pub_key_to_bytes(eph_pk);
    for i in 0..64 {
        encrypted_bytes[64 + i] = eph_pk_bytes[i];
    }
    for i in 0..48 {
        encrypted_bytes[128 + i] = incoming_header_ciphertext[i];
        encrypted_bytes[176 + i] = outgoing_Header_ciphertext[i];
    }
    for i in 0..176 {
        encrypted_bytes[224 + i] = outgoing_body_ciphertext[i];
    }
    // Then we fill in the rest as the incoming body ciphertext
    let size = M - 400;
    assert_eq(size, incoming_body_ciphertext.len(), "ciphertext length mismatch");
    for i in 0..size {
        encrypted_bytes[400 + i] = incoming_body_ciphertext[i];
    }

    // Current unoptimized size of the encrypted log
    // incoming_tag (32 bytes)
    // outgoing_tag (32 bytes)
    // eph_pk (64 bytes)
    // incoming_header (48 bytes)
    // outgoing_header (48 bytes)
    // outgoing_body (176 bytes)
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

fn compute_ivpk_app(ivpk: Point, contract_address: AztecAddress) -> Point {
    // It is useless to compute this, it brings no value to derive fully.
    // Issue(#6955)
    ivpk
    /*
    // @todo Just setting infinite to false, but it should be checked.
    // for example user could define ivpk = infinity using the registry
    assert((ivpk.x != 0) & (ivpk.y != 0), "ivpk is infinite");

    let i = fr_to_fq(poseidon2_hash([contract_address.to_field(), ivpk.x, ivpk.y, GENERATOR_INDEX__IVSK_M]));
    let I = derive_public_key(i);

    let embed_I = Point { x: I.x, y: I.y, is_infinite: false };
    let embed_ivpk = Point { x: ivpk.x, y: ivpk.y, is_infinite: false };

    let embed_result = embedded_curve_add(embed_I, embed_ivpk);

    Point { x: embed_result.x, embed_result.y)*/
}

mod test {
    use crate::{
        encrypted_logs::payload::compute_encrypted_note_log,
        test::mocks::mock_note::{MockNoteBuilder, MockNote}
    };
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
        OracleMock::mock("getRandomField").returns(eph_sk);

        let log: [u8; 512] = compute_encrypted_note_log(contract_address, storage_slot, ovsk_app, ovpk_m, ivpk_m, note);

        let expected_encrypted_note_log = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 70, 12, 14, 67, 77, 132, 110, 193, 234, 40, 110, 64, 144, 235, 86, 55, 111, 242, 123, 221, 193, 170, 202, 225, 216, 86, 84, 159, 112, 31, 167, 17, 160, 52, 217, 22, 191, 84, 175, 25, 133, 53, 220, 2, 251, 32, 105, 198, 147, 24, 131, 202, 112, 149, 136, 66, 205, 254, 3, 134, 195, 101, 73, 212, 19, 232, 42, 39, 191, 165, 183, 8, 7, 18, 118, 74, 69, 91, 146, 69, 16, 184, 101, 144, 48, 25, 190, 254, 181, 223, 24, 183, 175, 118, 159, 176, 135, 62, 255, 169, 124, 170, 3, 92, 81, 122, 107, 65, 125, 95, 97, 110, 198, 200, 74, 147, 217, 93, 23, 227, 84, 59, 15, 75, 108, 122, 49, 230, 228, 246, 207, 173, 7, 60, 16, 74, 236, 201, 102, 237, 48, 179, 223, 191, 223, 248, 78, 167, 61, 203, 25, 114, 223, 58, 60, 180, 255, 116, 170, 136, 173, 178, 40, 2, 125, 229, 20, 220, 82, 28, 191, 147, 133, 137, 1, 45, 243, 229, 140, 115, 165, 150, 154, 96, 22, 120, 223, 237, 213, 182, 252, 192, 8, 132, 43, 21, 56, 243, 116, 144, 182, 75, 16, 30, 222, 222, 60, 205, 147, 214, 53, 41, 62, 53, 16, 147, 117, 72, 169, 220, 125, 208, 210, 12, 186, 204, 143, 114, 50, 101, 80, 157, 55, 73, 11, 125, 27, 236, 202, 2, 242, 81, 153, 206, 165, 215, 215, 116, 50, 93, 177, 236, 240, 26, 164, 68, 160, 91, 136, 250, 39, 177, 22, 37, 38, 82, 248, 55, 125, 108, 38, 234, 209, 151, 69, 187, 150, 105, 120, 187, 236, 212, 161, 78, 22, 6, 231, 229, 110, 248, 35, 22, 254, 17, 32, 191, 75, 35, 252, 227, 111, 46, 146, 52, 21, 171, 105, 20, 112, 233, 143, 177, 99, 185, 193, 164, 246, 192, 62, 234, 175, 198, 157, 31, 230, 233, 237, 26, 65, 24, 65, 72, 220, 247, 223, 61, 75, 246, 144, 94, 99, 132, 235, 149, 93, 3, 115, 131, 132, 250, 238, 138, 194, 233, 144, 156, 140, 1, 42, 44, 12, 214, 94, 137, 130, 56, 105, 149, 124, 81, 178, 1, 73, 79, 156, 26, 65, 163, 18, 152, 116, 138, 128, 158, 63, 151, 205, 151, 73, 68, 173, 220, 126, 213, 72, 112, 237, 15, 235, 178, 217, 122, 146, 224, 89, 213, 217, 34, 172, 4, 164, 40, 102, 220, 174, 222, 250, 221, 149, 238, 174, 97, 65, 183, 255, 168, 139, 67, 122, 124, 41, 89, 147, 255, 109, 57, 197, 150, 174, 186, 221, 50, 19, 216, 14, 100, 176
        ];
        for i in 0..expected_encrypted_note_log.len() {
            assert_eq(log[i], expected_encrypted_note_log[i]);
        }
        assert_eq(expected_encrypted_note_log.len(), log.len());
    }
}
