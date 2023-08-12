mod storage;


contract Tets {
    use dep::aztec::context::PrivateContext;
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateCircuitPublicInputs;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::log::emit_unencrypted_log;

    use dep::value_note::{
        utils::{send_note, spend_notes},
    };
    use crate::storage::Storage;

    // Constructs the contract and sets `initial_supply` which is fully owned by `owner`.
    #[aztec(private)]
    fn constructor(
        //*********************************/
        initial_supply: Field, 
        owner: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {

        let storage = Storage::init();

        // Insert new note to a set of user notes and emit the newly created encrypted note preimage via oracle call.
        let owner_balance = storage.balances.at(owner);
        if (initial_supply != 0) {
            send_note(&mut context, owner_balance, initial_supply, owner);
            emit_unencrypted_log(&mut context, "Balance set in constructor");
        }

    }

}