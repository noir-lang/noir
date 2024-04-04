contract Counter {
    // docs:start:imports
    use dep::aztec::prelude::{AztecAddress, Map};
    use dep::aztec::context::Context;
    use dep::value_note::{balance_utils, value_note::{ValueNote, VALUE_NOTE_LEN}};
    use dep::easy_private_state::EasyPrivateUint;
    // docs:end:imports

    // docs:start:storage_struct
    #[aztec(storage)]
    struct Storage {
        counters: Map<AztecAddress, EasyPrivateUint>,
    }
    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(private)]
    #[aztec(initializer)]
    // We can name our initializer anything we want as long as it's marked as aztec(initializer)
    fn initialize(headstart: u64, owner: AztecAddress) {
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
}
