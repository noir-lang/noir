use dep::protocol_types::address::AztecAddress;

struct UnconstrainedContext {
    block_number: u32,
    contract_address: AztecAddress,    
}

impl UnconstrainedContext {
    fn new() -> Self {
        // We could call these oracles on the getters instead of at creation, which makes sense given that they might
        // not even be accessed. However any performance gains are minimal, and we'd rather fail early if a user
        // incorrectly attempts to create an UnconstrainedContext in an environment in which these oracles are not
        // available.
        let block_number = block_number_oracle();
        let contract_address = contract_address_oracle();
        Self { block_number, contract_address }
    }

    fn block_number(self) -> u32 {
        self.block_number
    }

    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

#[oracle(getContractAddress)]
fn contract_address_oracle() -> AztecAddress {}

#[oracle(getBlockNumber)]
fn block_number_oracle() -> u32 {}
