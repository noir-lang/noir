contract Counter {
    // docs:start:imports
    use dep::aztec::prelude::{AztecAddress, Map};
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
    fn initialize(headstart: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let counters = storage.counters;
        counters.at(owner).add(headstart, owner, outgoing_viewer);
    }
    // docs:end:constructor

    // docs:start:increment
    #[aztec(private)]
    fn increment(owner: AztecAddress, outgoing_viewer: AztecAddress) {
        dep::aztec::oracle::debug_log::debug_log_format("Incrementing counter for owner {0}", [owner.to_field()]);
        let counters = storage.counters;
        counters.at(owner).add(1, owner, outgoing_viewer);
    }
    // docs:end:increment

    // docs:start:get_counter
    unconstrained fn get_counter(owner: AztecAddress) -> pub Field {
        let counters = storage.counters;
        balance_utils::get_balance(counters.at(owner).set)
    }
    // docs:end:get_counter
}
