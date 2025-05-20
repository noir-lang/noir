use crate::types::{
    abis::log::Log, address::AztecAddress, constants::CONTRACT_CLASS_LOG_SIZE_IN_FIELDS,
    traits::Empty,
};

pub(crate) struct ContractClassLog {
    pub(crate) log: Log<CONTRACT_CLASS_LOG_SIZE_IN_FIELDS>,
    pub(crate) contract_address: AztecAddress,
}

impl Empty for ContractClassLog {
    fn empty() -> Self {
        ContractClassLog { log: Log::empty(), contract_address: AztecAddress::empty() }
    }
}
