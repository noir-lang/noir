use dep::types::{
    abis::{global_variables::GlobalVariables, append_only_tree_snapshot::AppendOnlyTreeSnapshot},
    traits::{Empty, Serialize, Deserialize}, constants::CONSTANT_ROLLUP_DATA_LENGTH,
    utils::reader::Reader
};

struct ConstantRollupData {
    // Archive tree snapshot at the very beginning of the entire rollup.
    last_archive : AppendOnlyTreeSnapshot,

    vk_tree_root : Field,

    global_variables : GlobalVariables,
}

impl Eq for ConstantRollupData {
    fn eq(self, other : ConstantRollupData) -> bool {
        self.last_archive.eq(other.last_archive) &
        self.global_variables.eq(other.global_variables) &
        (self.vk_tree_root == other.vk_tree_root)
    }
}

impl Empty for ConstantRollupData {
    fn empty() -> Self {
        ConstantRollupData {
            last_archive: AppendOnlyTreeSnapshot::zero(),
            vk_tree_root: 0,
            global_variables: GlobalVariables::empty(),
        }
    }
}

impl Serialize<CONSTANT_ROLLUP_DATA_LENGTH> for ConstantRollupData {
  fn serialize(self) -> [Field; CONSTANT_ROLLUP_DATA_LENGTH] {
    let mut fields: BoundedVec<Field, CONSTANT_ROLLUP_DATA_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.last_archive.serialize());
    fields.push(self.vk_tree_root as Field);
    fields.extend_from_array(self.global_variables.serialize());

    assert_eq(fields.len(), CONSTANT_ROLLUP_DATA_LENGTH);

    fields.storage
  }
}

impl Deserialize<CONSTANT_ROLLUP_DATA_LENGTH> for ConstantRollupData {
  fn deserialize(fields: [Field; CONSTANT_ROLLUP_DATA_LENGTH]) -> ConstantRollupData {
    let mut reader = Reader::new(fields);
    let item = Self {
        last_archive: reader.read_struct(AppendOnlyTreeSnapshot::deserialize),
        vk_tree_root: reader.read(),
        global_variables: reader.read_struct(GlobalVariables::deserialize),
    };

    reader.finish();
    item
  }
}

#[test]
fn serialization_of_empty() {
    let item = ConstantRollupData::empty();
    let serialized = item.serialize();
    let deserialized = ConstantRollupData::deserialize(serialized);
    assert(item.eq(deserialized));
}
