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

    // docs:start:test_imports
    use dep::aztec::test::{helpers::{cheatcodes, test_environment::TestEnvironment}};
    use dep::aztec::protocol_types::storage::map::derive_storage_slot_in_map;
    use dep::aztec::note::note_getter::{MAX_NOTES_PER_PAGE, view_notes};
    use dep::aztec::note::note_viewer_options::NoteViewerOptions;
    // docs:end:test_imports

    // docs:start:txe_test_increment
    #[test]
    fn test_increment() {
        // Setup env, generate keys
        let mut env = TestEnvironment::new();
        let owner = env.create_account();
        let outgoing_viewer = env.create_account();
        let initial_value: Field = 5;
        env.impersonate(owner);

        // Deploy contract and initialize 
        let initializer = Counter::interface().initialize(initial_value as u64, owner, outgoing_viewer);
        let counter_contract = env.deploy_self("Counter").with_private_initializer(initializer);
        let contract_address = counter_contract.to_address();

        // docs:start:txe_test_read_notes
        // Read the stored value in the note
        env.impersonate(contract_address);
        let counter_slot = Counter::storage().counters.slot;
        let owner_slot = derive_storage_slot_in_map(counter_slot, owner);
        let mut options = NoteViewerOptions::new();
        let notes: BoundedVec<ValueNote, MAX_NOTES_PER_PAGE> = view_notes(owner_slot, options);
        let initial_note_value = notes.get(0).value;
        assert(
            initial_note_value == initial_value, f"Expected {initial_value} but got {initial_note_value}"
        );
        // docs:end:txe_test_read_notes

        // Increment the counter
        let increment_call_interface = Counter::at(contract_address).increment(owner, outgoing_viewer);
        env.call_private_void(increment_call_interface);
        // get_counter is an unconstrained function, so we call it directly (we're in the same module)
        let current_value_for_owner = get_counter(owner);
        let expected_current_value = initial_value + 1;
        assert(
            expected_current_value == current_value_for_owner, f"Expected {expected_current_value} but got {current_value_for_owner}"
        );
    }
    // docs:end:txe_test_increment
}
