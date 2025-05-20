use crate::{
    abis::log::Log, address::AztecAddress, constants::CONTRACT_CLASS_LOG_SIZE_IN_FIELDS,
    traits::Empty,
};

pub struct ContractClassLog {
    pub log: Log<CONTRACT_CLASS_LOG_SIZE_IN_FIELDS>,
    pub contract_address: AztecAddress,
}

impl Empty for ContractClassLog {
    fn empty() -> Self {
        ContractClassLog { log: Log::empty(), contract_address: AztecAddress::empty() }
    }
}
