use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    context::PrivateContext,
    constants_gen::MAX_READ_REQUESTS_PER_CALL,
    state_vars::set::Set,
    log::emit_encrypted_log
};
use dep::aztec::types::address::AztecAddress;
use dep::aztec::oracle::{
    rand::rand,
    get_secret_key::get_secret_key,
    get_public_key::get_public_key,
};

use dep::safe_math::SafeU120;
use dep::std::option::Option;

global TOKEN_NOTE_LEN: Field = 3; // 3 plus a header.

struct TokenNote {
    // the amount of tokens in the note
    amount: SafeU120,
    // the provider of secrets for the nullifier. The owner (recipient) to ensure that the note 
    // can be privately spent. When nullifier secret and encryption private key is same 
    // we can simply use the owner for this one.
    owner: AztecAddress,
    // randomness of the note to hide contents.
    randomness: Field,
    // the note header (contract_address, nonce, storage_slot)
    // included in the note such that it becomes part of encrypted logs for later use.
    header: NoteHeader,
}

impl TokenNote {
    pub fn new(amount: SafeU120, owner: AztecAddress) -> Self {
        Self {
            amount,
            owner,
            randomness: rand(),
            header: NoteHeader::empty(),
        }
    }

    pub fn serialize(self) -> [Field; TOKEN_NOTE_LEN] {
        [self.amount.value as Field, self.owner.address, self.randomness]
    }

    pub fn deserialize(preimage: [Field; TOKEN_NOTE_LEN]) -> Self {
        Self {
            amount: SafeU120::new(preimage[0]),
            owner: AztecAddress::new(preimage[1]),
            randomness: preimage[2],
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        dep::std::hash::pedersen([
            self.amount.value as Field, 
            self.owner.address as Field,
            self.randomness,
        ])[0]
    }

    // docs:start:nullifier
    pub fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(TokenNoteMethods, self);
        let secret = get_secret_key(self.owner.address);
        // TODO(#1205) Should use a non-zero generator index.
        dep::std::hash::pedersen([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ])[0]
    }
    // docs:end:nullifier

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }


  pub fn emit_encrypted(
      self: &mut Self,
      context: &mut PrivateContext,
      storage_slot: Field,
  ) {
      // We only bother inserting the note if non-empty to save funds on gas.
      if !self.amount.is_zero() {
          // docs:start:encrypted
          let application_contract_address = (*context).this_address();
          let encryption_pub_key = get_public_key(self.owner.address);
          let encrypted_data = (*self).serialize();

          emit_encrypted_log(
              context,
              application_contract_address,
              storage_slot,
              encryption_pub_key,
              encrypted_data,
          );
          // docs:end:encrypted
      }
  }
}

fn deserialize(preimage: [Field; TOKEN_NOTE_LEN]) -> TokenNote {
    TokenNote::deserialize(preimage)
}

fn serialize(note: TokenNote) -> [Field; TOKEN_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: TokenNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: TokenNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: TokenNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut TokenNote, header: NoteHeader) {
    note.set_header(header)
}

global TokenNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};