use crate::{
    abis::side_effect::{counted::Counted, scoped::Scoped},
    address::AztecAddress,
    constants::LOG_HASH_LENGTH,
    traits::{Deserialize, Empty, Serialize},
};

pub struct LogHash {
    pub value: Field,
    pub length: u32,
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
    pub fn count(self, counter: u32) -> Counted<LogHash> {
        Counted::new(self, counter)
    }

    pub fn scope(self, contract_address: AztecAddress) -> Scoped<LogHash> {
        Scoped::new(self, contract_address)
    }
}

impl Scoped<Counted<LogHash>> {
    pub fn expose_to_public(self) -> Scoped<LogHash> {
        self.inner.inner.scope(self.contract_address)
    }
}
