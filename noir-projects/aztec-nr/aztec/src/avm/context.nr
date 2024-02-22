use dep::protocol_types::address::{AztecAddress, EthAddress};

// Getters that will be converted by the transpiler into their
// own opcodes
// No new function as this struct is entirely static getters

struct AvmContext {}

impl AvmContext {
    #[oracle(address)]
    pub fn address() -> AztecAddress {}

    #[oracle(storageAddress)]
    pub fn storage_address() -> AztecAddress {}

    #[oracle(origin)]
    pub fn origin() -> AztecAddress {}

    #[oracle(sender)]
    pub fn sender() -> AztecAddress {}

    #[oracle(portal)]
    pub fn portal() -> EthAddress {}

    #[oracle(feePerL1Gas)]
    pub fn fee_per_l1_gas() -> Field {}

    #[oracle(feePerL2Gas)]
    pub fn fee_per_l2_gas() -> Field {}

    #[oracle(feePerDaGas)]
    pub fn fee_per_da_gas() -> Field {}

    #[oracle(chainId)]
    pub fn chain_id() -> Field {}

    #[oracle(version)]
    pub fn version() -> Field {}

    #[oracle(blockNumber)]
    pub fn block_number() -> Field {}

    #[oracle(timestamp)]
    pub fn timestamp() -> Field {}
    // #[oracle(contractCallDepth)]
    // pub fn contract_call_depth() -> Field {}
}
