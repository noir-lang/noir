contract Counter {
    // docs:start:imports
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::aztec::{
        context::{PrivateContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        state_vars::map::Map,
    };
    use dep::value_note::{
            balance_utils,
            value_note::{
                ValueNote,
                VALUE_NOTE_LEN,
            },
    };
    use dep::easy_private_state::easy_private_state::EasyPrivateUint;
    // docs:end:imports

    // docs:start:storage_struct

    struct Storage {
        counters: Map<AztecAddress, EasyPrivateUint>,
    }

    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(private)]
    fn constructor(headstart: u120, owner: AztecAddress) {
        let counters = storage.counters;
        counters.at(owner).add(headstart, owner);
    }
    // docs:end:constructor

    // docs:start:increment
    #[aztec(private)]
      fn increment(owner: AztecAddress) {
        let counters = storage.counters;
        counters.at(owner).add(1, owner);
    }
    // docs:end:increment

    // docs:start:get_counter
    unconstrained fn get_counter(owner: AztecAddress) -> pub Field {
        let counters = storage.counters;
        balance_utils::get_balance(counters.at(owner).set)
    }
    // docs:end:get_counter

    // docs:start:nullifier
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
    }
    // docs:end:nullifier
}
