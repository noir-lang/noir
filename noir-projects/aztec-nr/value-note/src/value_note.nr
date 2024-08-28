use dep::aztec::{
    generators::{Ga1 as G_amt, Ga2 as G_npk, Ga3 as G_rnd, G_slot},
    protocol_types::{
    address::AztecAddress, traits::{Deserialize, Serialize}, constants::GENERATOR_INDEX__NOTE_NULLIFIER,
    hash::poseidon2_hash_with_separator, point::{Point, POINT_LENGTH}
},
    note::{note_header::NoteHeader, note_interface::NoteInterface, utils::compute_note_hash_for_nullify},
    oracle::unsafe_rand::unsafe_rand, keys::getters::get_nsk_app, context::PrivateContext
};
use dep::std::{embedded_curve_ops::multi_scalar_mul, hash::from_field_unsafe};

global VALUE_NOTE_LEN: Field = 3; // 3 plus a header.
// VALUE_NOTE_LEN * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global VALUE_NOTE_BYTES_LEN: Field = 3 * 32 + 64;

trait OwnedNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self;
    fn get_amount(self) -> U128;
}

// docs:start:value-note-def
#[aztec(note)]
struct ValueNote {
    value: Field,
   // The nullifying public key hash is used with the nsk_app to ensure that the note can be privately spent.
    npk_m_hash: Field,
    randomness: Field,
}
// docs:end:value-note-def

impl NoteInterface<VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN> for ValueNote {
    // docs:start:nullifier

    fn compute_nullifier(self, context: &mut PrivateContext, note_hash_for_nullify: Field) -> Field {
        let secret = context.request_nsk_app(self.npk_m_hash);
        poseidon2_hash_with_separator([
            note_hash_for_nullify,
            secret
        ],
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        )
    }

    // docs:end:nullifier

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_nullify(self);
        let secret = get_nsk_app(self.npk_m_hash);
        poseidon2_hash_with_separator([
            note_hash_for_nullify,
            secret,
        ],
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        )
    }

    fn compute_note_hiding_point(self) -> Point {
        // We use the unsafe version because the multi_scalar_mul will constrain the scalars.
        let amount_scalar = from_field_unsafe(self.value);
        let npk_m_hash_scalar = from_field_unsafe(self.npk_m_hash);
        let randomness_scalar = from_field_unsafe(self.randomness);
        let slot_scalar = from_field_unsafe(self.header.storage_slot);
        // We compute the note hiding point as:
        //     `G_amt * amount + G_npk * npk_m_hash + G_rnd * randomness + G_slot * slot`
        // instead of using pedersen or poseidon2 because it allows us to privately add and subtract from amount
        // in public by leveraging homomorphism.
        multi_scalar_mul(
            [G_amt, G_npk, G_rnd, G_slot],
            [amount_scalar, npk_m_hash_scalar, randomness_scalar, slot_scalar]
        )
    }
}

impl ValueNote {
    pub fn new(value: Field, npk_m_hash: Field) -> Self {
        let randomness = unsafe_rand();
        let header = NoteHeader::empty();
        ValueNote { value, npk_m_hash, randomness, header }
    }

    // TODO: Merge this func with `compute_note_hiding_point`. I (benesjan) didn't do it in the initial PR to not have
    // to modify macros and all the related funcs in it.
    fn to_note_hiding_point(self) -> ValueNoteHidingPoint {
        ValueNoteHidingPoint::new(self.compute_note_hiding_point())
    }
}

impl Serialize<7> for ValueNote {
    /// The following method needed to be implemented because the note is passed as an argument to a contract function
    /// --> the serialize method is called by aztec-nr when computing an arguments hash.
    /// Note that when the note is about to be encrypted and emitted as a log the to_be_bytes function auto-implemented
    /// by aztec macros is called instead.
    fn serialize(self) -> [Field; 7] {
        let header = self.header.serialize();

        [self.value, self.npk_m_hash, self.randomness, header[0], header[1], header[2], header[3]]
    }
}

impl Eq for ValueNote {
    fn eq(self, other: Self) -> bool {
        (self.value == other.value) &
        (self.npk_m_hash == other.npk_m_hash) &
        (self.randomness == other.randomness)
    }
}

impl OwnedNote for ValueNote {
    fn new(value: U128, owner_npk_m_hash: Field) -> Self {
        Self {
            value: value.to_field(),
            npk_m_hash: owner_npk_m_hash,
            randomness: unsafe_rand(),
            header: NoteHeader::empty(),
        }
    }

    fn get_amount(self) -> U128 {
        U128::from_field(self.value)
    }
}

struct ValueNoteHidingPoint {
  inner: Point
}

impl ValueNoteHidingPoint {
    fn new(point: Point) -> Self {
        Self { inner: point }
    }

    fn add_value(&mut self, value: U128) {
        self.inner = multi_scalar_mul([G_amt], [from_field_unsafe(value.to_integer())]) + self.inner;
    }

    fn add_npk_m_hash(&mut self, npk_m_hash: Field) {
        self.inner = multi_scalar_mul([G_npk], [from_field_unsafe(npk_m_hash)]) + self.inner;
    }

    fn add_randomness(&mut self, randomness: Field) {
        self.inner = multi_scalar_mul([G_rnd], [from_field_unsafe(randomness)]) + self.inner;
    }

    fn add_slot(&mut self, slot: Field) {
        self.inner = multi_scalar_mul([G_slot], [from_field_unsafe(slot)]) + self.inner;
    }

    fn finalize(self) -> Field {
        self.inner.x
    }
}

impl Serialize<POINT_LENGTH> for ValueNoteHidingPoint {
    fn serialize(self) -> [Field; POINT_LENGTH] {
        self.inner.serialize()
    }
}
