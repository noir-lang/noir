//

contract AvmTest {
    // Libs
    use dep::aztec::protocol_types::address::{
        AztecAddress,
        EthAddress,
    };

    // avm lib
    use dep::aztec::avm::context::AvmContext;

    #[aztec(private)]
    fn constructor() {}

    // Public-vm macro will prefix avm to the function name for transpilation
    #[aztec(public-vm)]
    fn addArgsReturn(argA: Field, argB: Field) -> pub Field {
        argA + argB
    }

    /************************************************************************
     * AvmContext functions
     ************************************************************************/
    #[aztec(public-vm)]
    fn getAddress() -> pub AztecAddress {
        AvmContext::address()
    }

    #[aztec(public-vm)]
    fn getStorageAddress() -> pub AztecAddress {
        AvmContext::storage_address()
    }

    #[aztec(public-vm)]
    fn getSender() -> pub AztecAddress {
        AvmContext::sender()
    }

    #[aztec(public-vm)]
    fn getOrigin() -> pub AztecAddress {
        AvmContext::origin()
    }

    #[aztec(public-vm)]
    fn getPortal() -> pub EthAddress {
        AvmContext::portal()
    }

    #[aztec(public-vm)]
    fn getFeePerL1Gas() -> pub Field {
        AvmContext::fee_per_l1_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerL2Gas() -> pub Field {
        AvmContext::fee_per_l2_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerDaGas() -> pub Field {
        AvmContext::fee_per_da_gas()
    }

    #[aztec(public-vm)]
    fn getChainId() -> pub Field {
        AvmContext::chain_id()
    }

    #[aztec(public-vm)]
    fn getVersion() -> pub Field {
        AvmContext::version()
    }

    #[aztec(public-vm)]
    fn getBlockNumber() -> pub Field {
        AvmContext::block_number()
    }

    #[aztec(public-vm)]
    fn getTimestamp() -> pub Field {
        AvmContext::timestamp()
    }

    // #[aztec(public-vm)]
    // fn getContractCallDepth() -> pub Field {
    //     AvmContext::contract_call_depth()
    // }

    // Function required for all contracts
    unconstrained fn compute_note_hash_and_nullifier(
        _contract_address: AztecAddress,
        _nonce: Field,
        _storage_slot: Field,
        _serialized_note: [Field; 1]
    ) -> pub [Field; 4] {
        [0, 0, 0, 0]
    }
}
