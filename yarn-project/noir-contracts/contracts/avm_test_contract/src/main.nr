//

contract AvmTest {
    // Libs
    use dep::aztec::protocol_types::{
        address::AztecAddress,
    };

    #[aztec(private)]
    fn constructor() {}

    // Function name prefix "avm_" flags it for transpilation
    unconstrained fn avm_addArgsReturn(argA: Field, argB: Field) -> pub Field {
        argA + argB
    }

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
