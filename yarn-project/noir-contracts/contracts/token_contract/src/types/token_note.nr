use dep::aztec::protocol_types::{
    address::AztecAddress,
    constants::{
        MAX_READ_REQUESTS_PER_CALL
    },
};
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    context::PrivateContext,
    state_vars::set::Set,
    log::emit_encrypted_log,
    hash::pedersen_hash,
};
use dep::aztec::oracle::{
    rand::rand,
    nullifier_key::get_nullifier_secret_key,
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
        [self.amount.value as Field, self.owner.to_field(), self.randomness]
    }

    pub fn deserialize(serialized_note: [Field; TOKEN_NOTE_LEN]) -> Self {
        Self {
            amount: SafeU120::new(serialized_note[0]),
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            self.amount.value as Field, 
            self.owner.to_field(),
            self.randomness,
        ],0)
    }

    // docs:start:nullifier
    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(TokenNoteMethods, self);
        let secret = context.request_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }
    // docs:end:nullifier

    pub fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(TokenNoteMethods, self);
        let secret = get_nullifier_secret_key(self.owner);
        // TODO(#1205) Should use a non-zero generator index.
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    // Broadcasts the note as an encrypted log on L1.
    pub fn broadcast(self, context: &mut PrivateContext, slot: Field) {
      // We only bother inserting the note if non-empty to save funds on gas.
      if !self.amount.is_zero() {
          let encryption_pub_key = get_public_key(self.owner);
          emit_encrypted_log(
              context,
              (*context).this_address(),
              slot,
              encryption_pub_key,
              self.serialize(),
          );
      }
  }
}

fn deserialize(serialized_note: [Field; TOKEN_NOTE_LEN]) -> TokenNote {
    TokenNote::deserialize(serialized_note)
}

fn serialize(note: TokenNote) -> [Field; TOKEN_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: TokenNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: TokenNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: TokenNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: TokenNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut TokenNote, header: NoteHeader) {
    note.set_header(header)
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: TokenNote) {
    note.broadcast(context, slot);
}

global TokenNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
