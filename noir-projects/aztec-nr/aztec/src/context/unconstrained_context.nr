use dep::protocol_types::{address::AztecAddress, traits::Deserialize};
use crate::oracle::{
    execution::{get_chain_id, get_version, get_contract_address, get_block_number},
    storage::{raw_storage_read, storage_read}
};

struct UnconstrainedContext {
    block_number: u32,
    contract_address: AztecAddress,
    version: Field,
    chain_id: Field,
}

impl UnconstrainedContext {
    unconstrained fn new() -> Self {
        // We could call these oracles on the getters instead of at creation, which makes sense given that they might
        // not even be accessed. However any performance gains are minimal, and we'd rather fail early if a user
        // incorrectly attempts to create an UnconstrainedContext in an environment in which these oracles are not
        // available.
        let block_number = get_block_number();
        let contract_address = get_contract_address();
        let chain_id = get_chain_id();
        let version = get_version();
        Self { block_number, contract_address, version, chain_id }
    }

    unconstrained fn at(contract_address: AztecAddress) -> Self {
        let block_number = get_block_number();
        let chain_id = get_chain_id();
        let version = get_version();
        Self { block_number, contract_address, version, chain_id }
    }

    fn block_number(self) -> u32 {
        self.block_number
    }

    fn this_address(self) -> AztecAddress {
        self.contract_address
    }

    fn version(self) -> Field {
        self.version
    }

    fn chain_id(self) -> Field {
        self.chain_id
    }

    unconstrained fn raw_storage_read<let N: u32>(self: Self, storage_slot: Field) -> [Field; N] {
        storage_read(self.this_address(), storage_slot, self.block_number())
    }

    unconstrained fn storage_read<T, let N: u32>(
        self,
        storage_slot: Field
    ) -> T where T: Deserialize<N> {
        T::deserialize(self.raw_storage_read(storage_slot))
    }
}
