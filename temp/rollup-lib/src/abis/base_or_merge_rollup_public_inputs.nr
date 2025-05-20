use dep::types::{
    abis::{constant_rollup_data::ConstantRollupData, sponge_blob::SpongeBlob},
    constants::BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH,
    partial_state_reference::PartialStateReference,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub(crate) global BASE_ROLLUP_TYPE: u32 = 0;
pub(crate) global MERGE_ROLLUP_TYPE: u32 = 1;

pub struct BaseOrMergeRollupPublicInputs {
    // rollup_type is either 0 (base) or 1 (merge)
    // TODO(Kev): Why is this a u32 instead of a u8/u16?
    pub(crate) rollup_type: u32,
    pub(crate) num_txs: u32,
    pub(crate) constants: ConstantRollupData,

    pub(crate) start: PartialStateReference,
    pub(crate) end: PartialStateReference,

    // A poseidon sponge of all tx effects to be opened in block root, then added to blob(s)
    // In sponge form so we can check left.end == right.start, rather than recreate a tree later
    pub(crate) start_sponge_blob: SpongeBlob,
    pub(crate) end_sponge_blob: SpongeBlob,

    // We hash public inputs to make them constant-sized (to then be opened on-chain)
    // u128 isn't safe if it's an input to the circuit (it won't automatically constrain the witness)
    // So we want to constrain it when casting these fields to u128

    pub(crate) out_hash: Field,

    pub(crate) accumulated_fees: Field,
    pub(crate) accumulated_mana_used: Field,
}

impl Empty for BaseOrMergeRollupPublicInputs {
    fn empty() -> Self {
        BaseOrMergeRollupPublicInputs {
            rollup_type: 0 as u32,
            num_txs: 0 as u32,
            constants: ConstantRollupData::empty(),
            start: PartialStateReference::empty(),
            end: PartialStateReference::empty(),
            start_sponge_blob: SpongeBlob::empty(),
            end_sponge_blob: SpongeBlob::empty(),
            out_hash: 0,
            accumulated_fees: 0,
            accumulated_mana_used: 0,
        }
    }
}

impl Eq for BaseOrMergeRollupPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.rollup_type == other.rollup_type)
            & (self.num_txs == other.num_txs)
            & (self.constants.eq(other.constants))
            & (self.start.eq(other.start))
            & (self.end.eq(other.end))
            & (self.start_sponge_blob.eq(other.start_sponge_blob))
            & (self.end_sponge_blob.eq(other.end_sponge_blob))
            & (self.out_hash == other.out_hash)
            & (self.accumulated_fees == other.accumulated_fees)
            & (self.accumulated_mana_used == other.accumulated_mana_used)
    }
}

impl Serialize<BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> for BaseOrMergeRollupPublicInputs {
    fn serialize(self) -> [Field; BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();

        fields.push(self.rollup_type as Field);
        fields.push(self.num_txs as Field);
        fields.extend_from_array(self.constants.serialize());
        fields.extend_from_array(self.start.serialize());
        fields.extend_from_array(self.end.serialize());
        fields.extend_from_array(self.start_sponge_blob.serialize());
        fields.extend_from_array(self.end_sponge_blob.serialize());
        fields.push(self.out_hash as Field);
        fields.push(self.accumulated_fees as Field);
        fields.push(self.accumulated_mana_used as Field);
        assert_eq(fields.len(), BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH);

        fields.storage()
    }
}

impl Deserialize<BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH> for BaseOrMergeRollupPublicInputs {
    fn deserialize(
        fields: [Field; BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH],
    ) -> BaseOrMergeRollupPublicInputs {
        let mut reader = Reader::new(fields);
        let item = Self {
            rollup_type: reader.read() as u32,
            num_txs: reader.read() as u32,
            constants: reader.read_struct(ConstantRollupData::deserialize),
            start: reader.read_struct(PartialStateReference::deserialize),
            end: reader.read_struct(PartialStateReference::deserialize),
            start_sponge_blob: reader.read_struct(SpongeBlob::deserialize),
            end_sponge_blob: reader.read_struct(SpongeBlob::deserialize),
            out_hash: reader.read(),
            accumulated_fees: reader.read(),
            accumulated_mana_used: reader.read(),
        };

        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty() {
    let item = BaseOrMergeRollupPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = BaseOrMergeRollupPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
