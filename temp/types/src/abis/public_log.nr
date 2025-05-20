use crate::{
    abis::{log::Log, side_effect::Scoped},
    address::AztecAddress,
    constants::{PUBLIC_LOG_LENGTH, PUBLIC_LOG_SIZE_IN_FIELDS},
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct PublicLog {
    pub log: Log<PUBLIC_LOG_SIZE_IN_FIELDS>,
    pub contract_address: AztecAddress,
}

impl Scoped<Log<PUBLIC_LOG_SIZE_IN_FIELDS>> for PublicLog {
    fn inner(self) -> Log<PUBLIC_LOG_SIZE_IN_FIELDS> {
        self.log
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Eq for PublicLog {
    fn eq(self, other: PublicLog) -> bool {
        (self.log == other.log) & (self.contract_address == other.contract_address)
    }
}

impl Empty for PublicLog {
    fn empty() -> Self {
        PublicLog { log: Log::empty(), contract_address: AztecAddress::empty() }
    }
}

impl Serialize<PUBLIC_LOG_LENGTH> for PublicLog {
    fn serialize(self) -> [Field; PUBLIC_LOG_LENGTH] {
        // Note: we ensure that the address of a public log goes first, unlike generic Scoped<T> impls
        array_concat([self.contract_address.to_field()], self.log.serialize())
    }
}

impl Deserialize<PUBLIC_LOG_LENGTH> for PublicLog {
    fn deserialize(fields: [Field; PUBLIC_LOG_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        Self {
            contract_address: reader.read_struct(AztecAddress::deserialize),
            log: reader.read_struct(Log::deserialize),
        }
    }
}

#[test]
fn serialization_of_empty_public_log() {
    let item = PublicLog::empty();
    let serialized = item.serialize();
    let deserialized = PublicLog::deserialize(serialized);
    assert(item.eq(deserialized));
}
