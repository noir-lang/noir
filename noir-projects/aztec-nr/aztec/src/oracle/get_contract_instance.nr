use dep::protocol_types::{address::AztecAddress, contract_instance::ContractInstance, constants::CONTRACT_INSTANCE_LENGTH};

#[oracle(getContractInstance)]
fn get_contract_instance_oracle(_address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH] {}

unconstrained fn get_contract_instance_internal(address: AztecAddress) -> [Field; CONTRACT_INSTANCE_LENGTH] {
    get_contract_instance_oracle(address)
}

pub fn get_contract_instance(address: AztecAddress) -> ContractInstance {
    let instance = ContractInstance::deserialize(get_contract_instance_internal(address));
    assert(instance.to_address().eq(address));
    instance
}
