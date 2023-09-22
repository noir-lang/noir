mod address_note;

// Sample escrow contract that stores a balance of a private token on behalf of an owner.
contract Escrow {
    use dep::std::option::Option;

    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        log::emit_encrypted_log,
        note::{
            note_getter_options::NoteGetterOptions,
            note_header::NoteHeader,
            utils as note_utils,
        },
        oracle::get_public_key::get_public_key,
        oracle::compute_selector::compute_selector,
        state_vars::set::Set,
    };

    use crate::address_note::{
        AddressNote,
        AddressNoteMethods,
        ADDRESS_NOTE_LEN,
    };

    struct Storage {
        owners: Set<AddressNote, ADDRESS_NOTE_LEN>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                owners: Set::new(context, 1, AddressNoteMethods),
            }
        }
    }

    // Creates a new instance
    // docs:start:constructor
    #[aztec(private)]
    fn constructor(
        owner: pub Field
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let this = context.this_address();

        // Create a new note and add it to the owners set.
        let mut note = AddressNote::new(owner, this);

        // Insert the owner into storage
        storage.owners.insert(&mut note);
        emit_encrypted_log(
            &mut context,
            this,
            storage.owners.storage_slot,
            get_public_key(this),
            note.serialize(),
        );
    }
    // docs:end:constructor

    // Withdraws balance. Requires that msg.sender is registered as an owner.
    #[aztec(private)]
    fn withdraw(
        token: Field,
        amount: Field,
        recipient: Field,
    ) {
        let this = context.this_address();
        let sender = context.msg_sender();
        let storage = Storage::init(Context::private(&mut context));
        
        // We don't remove note from the owners set. If a note exists, the owner and recipient are legit.
        let options = NoteGetterOptions::new().select(0, sender).select(1, this).set_limit(1);
        let notes = storage.owners.get_notes(options);
        let note = notes[0].unwrap_unchecked();
        // Filter is not constrained. We still need to check if the note is what we expected.
        assert(note.address == sender);
        assert(note.owner == this);

        let selector = compute_selector("transfer((Field),(Field),Field,Field)");
        let _callStackItem = context.call_private_function(
            token,
            selector,
            [this, recipient, amount, 0]
        );
    }

    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, preimage: [Field; ADDRESS_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader { contract_address, nonce, storage_slot };
        assert(storage_slot == 1);
        note_utils::compute_note_hash_and_nullifier(AddressNoteMethods, note_header, preimage)
    }
}
