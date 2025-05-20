use crate::{
    abis::{log::Log, side_effect::{Ordered, scoped::Scoped}},
    address::AztecAddress,
    constants::{PRIVATE_LOG_DATA_LENGTH, PRIVATE_LOG_SIZE_IN_FIELDS},
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_concat, reader::Reader},
};

pub type PrivateLog = Log<PRIVATE_LOG_SIZE_IN_FIELDS>;

pub struct PrivateLogData {
    pub log: PrivateLog,
    // The counter of the note hash this log is for. 0 if it does not link to a note hash.
    pub note_hash_counter: u32,
    pub counter: u32,
}

impl Ordered for PrivateLogData {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for PrivateLogData {
    fn eq(self, other: PrivateLogData) -> bool {
        (self.log == other.log)
            & (self.note_hash_counter == other.note_hash_counter)
            & (self.counter == other.counter)
    }
}

impl Empty for PrivateLogData {
    fn empty() -> Self {
        PrivateLogData { log: PrivateLog::empty(), note_hash_counter: 0, counter: 0 }
    }
}

impl Serialize<PRIVATE_LOG_DATA_LENGTH> for PrivateLogData {
    fn serialize(self) -> [Field; PRIVATE_LOG_DATA_LENGTH] {
        array_concat(
            self.log.serialize(),
            [self.note_hash_counter as Field, self.counter as Field],
        )
    }
}

impl Deserialize<PRIVATE_LOG_DATA_LENGTH> for PrivateLogData {
    fn deserialize(fields: [Field; PRIVATE_LOG_DATA_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        Self {
            log: reader.read_struct(PrivateLog::deserialize),
            note_hash_counter: reader.read_u32(),
            counter: reader.read_u32(),
        }
    }
}

impl PrivateLogData {
    pub fn scope(self, contract_address: AztecAddress) -> Scoped<Self> {
        Scoped { inner: self, contract_address }
    }
}

impl Ordered for Scoped<PrivateLogData> {
    fn counter(self) -> u32 {
        self.inner.counter
    }
}

impl Scoped<PrivateLogData> {
    pub fn expose_to_public(self) -> PrivateLog {
        self.inner.log
    }
}

#[test]
fn serialization_of_empty_private_log_data() {
    let item = PrivateLogData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateLogData::deserialize(serialized);
    assert(item.eq(deserialized));
}
