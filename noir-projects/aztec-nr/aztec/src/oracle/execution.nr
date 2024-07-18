use dep::protocol_types::address::AztecAddress;

#[oracle(getContractAddress)]
unconstrained fn get_contract_address_oracle() -> AztecAddress {}

#[oracle(getBlockNumber)]
unconstrained fn get_block_number_oracle() -> u32 {}

#[oracle(getChainId)]
unconstrained fn get_chain_id_oracle() -> Field {}

#[oracle(getVersion)]
unconstrained fn get_version_oracle() -> Field {}

unconstrained pub fn get_contract_address() -> AztecAddress {
    get_contract_address_oracle()
}

unconstrained pub fn get_block_number() -> u32 {
    get_block_number_oracle()
}

unconstrained pub fn get_chain_id() -> Field {
    get_chain_id_oracle()
}

unconstrained pub fn get_version() -> Field {
    get_version_oracle()
}
