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

    use dep::aztec::test::{helpers::{cheatcodes, test_environment::TestEnvironment}};
    use dep::aztec::protocol_types::storage::map::derive_storage_slot_in_map;
    use dep::aztec::note::note_getter::{MAX_NOTES_PER_PAGE, view_notes};
    use dep::aztec::note::note_viewer_options::NoteViewerOptions;

    #[test]
    fn test_initialize() {
        // Setup env, generate keys
        let mut env = TestEnvironment::new();
        let owner = env.create_account();
        let outgoing_viewer = env.create_account();

        // Deploy contract and initialize 
        let initializer = Counter::interface().initialize(5, owner, outgoing_viewer);
        let contract_address = env.deploy("@aztec/noir-contracts.js/Counter").with_private_initializer(initializer);
        // Read the stored value in the note

        cheatcodes::set_contract_address(contract_address);
        let counter_slot = Counter::storage().counters.slot;
        let owner_slot = derive_storage_slot_in_map(counter_slot, owner);
        let mut options = NoteViewerOptions::new();
        let notes: BoundedVec<ValueNote, MAX_NOTES_PER_PAGE> = view_notes(owner_slot, options);
        assert(notes.get(0).value == 5);
    }
}
