use dep::protocol_types::address::{
    AztecAddress,
    EthAddress,
};

// Getters that will be converted by the transpiler into their
// own opcodes
struct AVMContext {}

impl AVMContext {
    // Empty new function enables retaining context.<value> syntax
    pub fn new() -> Self {
        Self {}
    }

    #[oracle(address)]
    pub fn address(self) -> AztecAddress {}

    #[oracle(storageAddress)]
    pub fn storage_address(self) -> AztecAddress {}

    #[oracle(origin)]
    pub fn origin(self) -> AztecAddress {}

    #[oracle(sender)]
    pub fn sender(self) -> AztecAddress {}

    #[oracle(portal)]
    pub fn portal(self) -> EthAddress {}

    #[oracle(feePerL1Gas)]
    pub fn fee_per_l1_gas(self) -> Field {}

    #[oracle(feePerL2Gas)]
    pub fn fee_per_l2_gas(self) -> Field {}

    #[oracle(feePerDaGas)]
    pub fn fee_per_da_gas(self) -> Field {}

    #[oracle(chainId)]
    pub fn chain_id(self) -> Field {}

    #[oracle(version)]
    pub fn version(self) -> Field {}

    #[oracle(blockNumber)]
    pub fn block_number(self) -> Field {}

    #[oracle(timestamp)]
    pub fn timestamp(self) -> Field {}

    // #[oracle(contractCallDepth)]
    // pub fn contract_call_depth(self) -> Field {}
}
