use dep::aztec::prelude::{AztecAddress, FunctionSelector, NoteHeader, NoteInterface, NoteGetterOptions, PrivateContext};

use dep::aztec::{
    note::utils::compute_note_hash_for_consumption, keys::getters::get_nsk_app,
    protocol_types::{constants::GENERATOR_INDEX__NOTE_NULLIFIER, grumpkin_point::GrumpkinPoint, hash::poseidon2_hash}
};

global ECDSA_PUBLIC_KEY_NOTE_LEN: Field = 5;
// ECDSA_PUBLIC_KEY_NOTE_LEN * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global ECDSA_PUBLIC_KEY_NOTE_BYTES_LEN: Field = 5 * 32 + 64;

// Stores an ECDSA public key composed of two 32-byte elements
// TODO: Do we need to include a nonce, in case we want to read/nullify/recreate with the same pubkey value?
#[aztec(note)]
struct EcdsaPublicKeyNote {
    x: [u8; 32],
    y: [u8; 32],
    // We store the npk_m_hash only to get the secret key to compute the nullifier
    npk_m_hash: Field,
}

impl NoteInterface<ECDSA_PUBLIC_KEY_NOTE_LEN, ECDSA_PUBLIC_KEY_NOTE_BYTES_LEN> for EcdsaPublicKeyNote {
    // Cannot use the automatic serialization since x and y don't fit. Serialize the note as 5 fields where:
    // [0] = x[0..31] (upper bound excluded)
    // [1] = x[31]
    // [2] = y[0..31]
    // [3] = y[31]
    // [4] = npk_m_hash
    fn serialize_content(self) -> [Field; ECDSA_PUBLIC_KEY_NOTE_LEN] {
        let mut x: Field = 0;
        let mut y: Field = 0;
        let mut mul: Field = 1;

        for i in 1..32 {
          let byte_x: Field = self.x[31 - i] as Field;
          x = x + (byte_x * mul);
          let byte_y: Field = self.y[31 - i] as Field;
          y = y + (byte_y * mul);
          mul *= 256;
        }

        let last_x = self.x[31] as Field;
        let last_y = self.y[31] as Field;
        
        [x, last_x, y, last_y, self.npk_m_hash]
    }

    // Cannot use the automatic deserialization for the aforementioned reasons
    fn deserialize_content(serialized_note: [Field; ECDSA_PUBLIC_KEY_NOTE_LEN]) -> EcdsaPublicKeyNote {
        let mut x: [u8; 32] = [0; 32];
        let mut y: [u8; 32] = [0; 32];

        let part_x = serialized_note[0].to_be_bytes(32);
        for i in 0..31 {
            x[i] = part_x[i + 1];
        }
        x[31] = serialized_note[1].to_be_bytes(32)[31];

        let part_y = serialized_note[2].to_be_bytes(32);
        for i in 0..31 {
            y[i] = part_y[i + 1];
        }
        y[31] = serialized_note[3].to_be_bytes(32)[31];

        EcdsaPublicKeyNote { x, y, npk_m_hash: serialized_note[4], header: NoteHeader::empty() }
    }

   fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nsk_app(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nsk_app(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field, ovpk_m: GrumpkinPoint, ivpk_m: GrumpkinPoint) {
        context.encrypt_and_emit_note(
            (*context).this_address(),
            slot,
            ovpk_m,
            ivpk_m,
            self,
        );
    }
}

impl EcdsaPublicKeyNote {
    pub fn new(x: [u8; 32], y: [u8; 32], npk_m_hash: Field) -> Self {
        EcdsaPublicKeyNote { x, y, npk_m_hash, header: NoteHeader::empty() }
    }
}
