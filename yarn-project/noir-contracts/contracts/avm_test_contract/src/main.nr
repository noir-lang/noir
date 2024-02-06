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
        context.address()
    }

    #[aztec(public-vm)]
    fn getStorageAddress() -> pub AztecAddress {
        context.storage_address()
    }

    #[aztec(public-vm)]
    fn getSender() -> pub AztecAddress {
        context.sender()
    }

    #[aztec(public-vm)]
    fn getOrigin() -> pub AztecAddress {
        context.origin()
    }

    #[aztec(public-vm)]
    fn getPortal() -> pub EthAddress {
        context.portal()
    }

    #[aztec(public-vm)]
    fn getFeePerL1Gas() -> pub Field {
        context.fee_per_l1_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerL2Gas() -> pub Field {
        context.fee_per_l2_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerDaGas() -> pub Field {
        context.fee_per_da_gas()
    }

    #[aztec(public-vm)]
    fn getChainId() -> pub Field {
        context.chain_id()
    }

    #[aztec(public-vm)]
    fn getVersion() -> pub Field {
        context.version()
    }

    #[aztec(public-vm)]
    fn getBlockNumber() -> pub Field {
        context.block_number()
    }

    #[aztec(public-vm)]
    fn getTimestamp() -> pub Field {
        context.timestamp()
    }

    // #[aztec(public-vm)]
    // fn getContractCallDepth() -> pub Field {
    //     context.contract_call_depth()
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
