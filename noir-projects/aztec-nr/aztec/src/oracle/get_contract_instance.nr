use dep::protocol_types::{
    address::AztecAddress, contract_instance::ContractInstance, utils::arr_copy_slice,
    constants::CONTRACT_INSTANCE_LENGTH, utils::reader::Reader
};

#[oracle(getContractInstance)]
fn get_contract_instance_oracle(_address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH] {}

// Returns a ContractInstance plus a boolean indicating whether the instance was found.
#[oracle(avmOpcodeGetContractInstance)]
fn get_contract_instance_oracle_avm(_address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH + 1] {}

unconstrained fn get_contract_instance_internal(address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH] {
    get_contract_instance_oracle(address)
}

unconstrained fn get_contract_instance_internal_avm(address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH + 1] {
    get_contract_instance_oracle_avm(address)
}

pub fn get_contract_instance(address: AztecAddress) -> ContractInstance {
    let instance = ContractInstance::deserialize(get_contract_instance_internal(address));
    assert(instance.to_address().eq(address));
    instance
}

pub fn get_contract_instance_avm(address: AztecAddress) -> Option<ContractInstance> {
    let mut reader = Reader::new(get_contract_instance_internal_avm(address));
    let found = reader.read();
    if found == 0 {
        Option::none()
    } else {
        Option::some(reader.read_struct(ContractInstance::deserialize))
    }
}
