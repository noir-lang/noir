use dep::aztec::{
    keys::getters::get_ivpk_m_with_npk_m_hash,
    prelude::{AztecAddress, NoteHeader, NoteInterface, PrivateContext},
    protocol_types::constants::GENERATOR_INDEX__NOTE_NULLIFIER,
    note::utils::compute_note_hash_for_consumption, hash::poseidon2_hash,
    oracle::{unsafe_rand::unsafe_rand, nullifier_key::get_nsk_app_with_npk_m_hash}
};

trait OwnedNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self;
    fn get_amount(self) -> U128;
    fn get_owner_npk_m_hash(self) -> Field;
}

global TOKEN_NOTE_LEN: Field = 3; // 3 plus a header.

#[aztec(note)]
struct TokenNote {
    // The amount of tokens in the note
    amount: U128,
    // The nullifying public key hash of the person who owns the note. 
    // This is used with the app_nullifier_secret_key to ensure that the note can be privately spent.
    npk_m_hash: Field,
    // Randomness of the note to hide its contents
    randomness: Field,
}

impl NoteInterface<TOKEN_NOTE_LEN> for TokenNote {
    // docs:start:nullifier
    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nsk_app_with_npk_m_hash(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }
    // docs:end:nullifier

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nsk_app_with_npk_m_hash(self.npk_m_hash);
        poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ])
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
      // We only bother inserting the note if non-empty to save funds on gas.
      // TODO: (#5901) This will be changed a lot, as it should use the updated encrypted log format
      if !(self.amount == U128::from_integer(0)) {
          let ivpk_m = get_ivpk_m_with_npk_m_hash(self.npk_m_hash);
          context.emit_encrypted_log(
              (*context).this_address(),
              slot,
              Self::get_note_type_id(),
              ivpk_m,
              self.serialize_content(),
          );
      }
    }
}

impl OwnedNote for TokenNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self {
        Self {
            amount,
            npk_m_hash: owner_npk_m_hash,
            randomness: unsafe_rand(),
            header: NoteHeader::empty(),
        }
    }

    fn get_amount(self) -> U128 {
        self.amount
    }

    fn get_owner_npk_m_hash(self) -> Field {
        self.npk_m_hash
    }
}
