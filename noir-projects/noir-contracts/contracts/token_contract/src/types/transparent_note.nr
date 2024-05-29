// docs:start:token_types_all
use dep::aztec::{
    note::{note_getter_options::PropertySelector, utils::compute_note_hash_for_consumption},
    prelude::{NoteHeader, NoteInterface, PrivateContext},
    protocol_types::{constants::GENERATOR_INDEX__NOTE_NULLIFIER, grumpkin_point::GrumpkinPoint, hash::poseidon2_hash}
};

global TRANSPARENT_NOTE_LEN: Field = 2;
// TRANSPARENT_NOTE_LEN * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global TRANSPARENT_NOTE_BYTES_LEN: Field = 2 * 32 + 64;

// Transparent note represents a note that is created in the clear (public execution), but can only be spent by those
// that know the preimage of the "secret_hash" (the secret). This is typically used when shielding a token balance.
// Owner of the tokens provides a "secret_hash" as an argument to the public "shield" function and then the tokens
// can be redeemed in private by presenting the preimage of the "secret_hash" (the secret).
#[aztec(note)]
struct TransparentNote {
    amount: Field,
    secret_hash: Field,
}

struct TransparentNoteProperties {
    amount: PropertySelector,
    secret_hash: PropertySelector,
}

impl NoteInterface<TRANSPARENT_NOTE_LEN, TRANSPARENT_NOTE_BYTES_LEN> for TransparentNote {

    // Custom serialization to avoid disclosing the secret field
    fn serialize_content(self) -> [Field; TRANSPARENT_NOTE_LEN] {
        [self.amount, self.secret_hash]
    }

    // Custom deserialization since we don't have access to the secret plaintext
    fn deserialize_content(serialized_note: [Field; TRANSPARENT_NOTE_LEN]) -> Self {
        TransparentNote {
            amount: serialized_note[0],
            secret_hash: serialized_note[1],
            header: NoteHeader::empty(),
        }
    }

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): Ensure nullifier collisions are prevented
    fn compute_nullifier(self, _context: &mut PrivateContext) -> Field {
        self.compute_nullifier_without_context()
    }

    // Computing a nullifier in a transparent note is not guarded by making secret a part of the nullifier preimage (as
    // is common in other cases) and instead is guarded by the functionality of "redeem_shield" function. There we do
    // the following:
    //      1) We pass the secret as an argument to the function and use it to compute a secret hash,
    //      2) we fetch a note via the "get_notes" oracle which accepts the secret hash as an argument,
    //      3) the "get_notes" oracle constrains that the secret hash in the returned note matches the one computed in
    //         circuit. 
    // This achieves that the note can only be spent by the party that knows the secret.
    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        poseidon2_hash([
            note_hash_for_nullify,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    fn broadcast(self, context: &mut PrivateContext, slot: Field, ovpk_m: GrumpkinPoint, ivpk_m: GrumpkinPoint) {
        assert(false, "TransparentNote does not support broadcast");
    }
}

impl TransparentNote {
    // CONSTRUCTORS
    pub fn new(amount: Field, secret_hash: Field) -> Self {
        TransparentNote { amount, secret_hash, header: NoteHeader::empty() }
    }

    // CUSTOM FUNCTIONS FOR THIS NOTE TYPE
    // Custom serialization forces us to manually create the metadata struct and its getter
    pub fn properties() -> TransparentNoteProperties {
        TransparentNoteProperties {
            amount: PropertySelector { index: 0, offset: 0, length: 32 },
            secret_hash: PropertySelector { index: 1, offset: 0, length: 32 }
        }
    }
}
// docs:end:token_types_all
