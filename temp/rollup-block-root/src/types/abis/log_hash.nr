use crate::types::{
    abis::side_effect::{counted::Counted, scoped::Scoped},
    address::AztecAddress,
    constants::LOG_HASH_LENGTH,
    traits::{Deserialize, Empty, Serialize},
};

pub(crate) struct LogHash {
    pub(crate) value: Field,
    pub(crate) length: u32,
}

impl Eq for LogHash {
    fn eq(self, other: LogHash) -> bool {
        (self.value == other.value) & (self.length == other.length)
    }
}

impl Empty for LogHash {
    fn empty() -> Self {
        LogHash { value: 0, length: 0 }
    }
}

impl Serialize<LOG_HASH_LENGTH> for LogHash {
    fn serialize(self) -> [Field; LOG_HASH_LENGTH] {
        [self.value, self.length as Field]
    }
}

impl Deserialize<LOG_HASH_LENGTH> for LogHash {
    fn deserialize(values: [Field; LOG_HASH_LENGTH]) -> Self {
        Self { value: values[0], length: values[1] as u32 }
    }
}

impl LogHash {
    pub(crate) fn count(self, counter: u32) -> Counted<LogHash> {
        Counted::new(self, counter)
    }

    pub(crate) fn scope(self, contract_address: AztecAddress) -> Scoped<LogHash> {
        Scoped::new(self, contract_address)
    }
}

impl Scoped<Counted<LogHash>> {
    pub(crate) fn expose_to_public(self) -> Scoped<LogHash> {
        self.inner.inner.scope(self.contract_address)
    }
}
