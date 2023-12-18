contract Blank {
    use dep::aztec::{
        abi,
        oracle::{
            get_public_key::get_public_key,
        },
    };

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn getPublicKey(
        address: Field,
    ) -> [Field; 2]{
        let pub_key = get_public_key(address);
        
        [pub_key.x, pub_key.y]
    }

    // A function which needs to be implemented by every contract working with storage. Replace it's content with your
    // own logic once you start working with private storage.
    // TODO: Remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented.
    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, serialized_note: [Field; 0]) -> [Field; 4] {
        [0, 0, 0, 0]
    }
}
