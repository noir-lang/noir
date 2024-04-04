// A contract used for running benchmarks.
// We should try to change this contract as little as possible, since any modification
// would alter the metrics we're capturing in the benchmarks, and we want to keep the
// subject being tested as unmodified as possible so we can detect metric changes that

contract Benchmarking {
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteGetterOptions, emit_unencrypted_log, Map,
        PublicMutable, PrivateSet
    };
    use dep::value_note::{utils::{increment, decrement}, value_note::ValueNote};

    use dep::aztec::{context::Context};

    #[aztec(storage)]
    struct Storage {
        notes: Map<AztecAddress, PrivateSet<ValueNote>>,
        balances: Map<AztecAddress, PublicMutable<Field>>,
    }

    // Creates a new value note for the target owner. Use this method to seed an initial set of notes.
    #[aztec(private)]
    fn create_note(owner: AztecAddress, value: Field) {
        increment(storage.notes.at(owner), value, owner);
    }

    // Deletes a note at a specific index in the set and creates a new one with the same value.
    // We explicitly pass in the note index so we can ensure we consume different notes when sending
    // multiple txs that will land on the same block.
    // See https://discourse.aztec.network/t/utxo-concurrency-issues-for-private-state/635
    // by @rahul-kothari for a full explanation on why this is needed.
    #[aztec(private)]
    fn recreate_note(owner: AztecAddress, index: u32) {
        let owner_notes = storage.notes.at(owner);
        let getter_options = NoteGetterOptions::new().set_limit(1).set_offset(index);
        let notes = owner_notes.get_notes(getter_options);
        let note = notes[0].unwrap_unchecked();
        owner_notes.remove(note);
        increment(owner_notes, note.value, owner);
    }

    // Reads and writes to public storage and enqueues a call to another public function.
    #[aztec(public)]
    fn increment_balance(owner: AztecAddress, value: Field) {
        let current = storage.balances.at(owner).read();
        storage.balances.at(owner).write(current + value);
        let _callStackItem1 = context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("broadcast((Field))"),
            [owner.to_field()]
        );
    }

    // Emits a public log.
    #[aztec(public)]
    fn broadcast(owner: AztecAddress) {
        emit_unencrypted_log(&mut context, storage.balances.at(owner).read());
    }
}
