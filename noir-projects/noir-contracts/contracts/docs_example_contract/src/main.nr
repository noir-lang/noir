mod options;
mod types;

// Following is a very simple game to show case use of PrivateMutable in as minimalistic way as possible
// It also serves as an e2e test that you can read and then replace the PrivateMutable in the same call
// (tests ordering in the circuit)

// you have a card (PrivateMutable). Anyone can create a bigger card. Whoever is bigger will be the leader.
// it also has dummy methods and other examples used for documentation e.g. 
// how to create custom notes, a custom struct for public state, a custom note that may be unencrypted
// also has `options.nr` which shows various ways of using `NoteGetterOptions` to query notes
// it also shows what our macros do behind the scenes!

contract DocsExample {
    // how to import dependencies defined in your workspace
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteGetterOptions, NoteViewerOptions,
        PrivateContext, Map, PublicMutable, PublicImmutable, PrivateMutable, PrivateImmutable,
        PrivateSet, SharedImmutable
    };
    use dep::aztec::{note::note_getter_options::Comparator, context::{PublicContext, Context}};
    // how to import methods from other files/folders within your workspace
    use crate::options::create_account_card_getter_options;
    use crate::types::{card_note::{CardNote, CARD_NOTE_LEN}, leader::Leader};

    struct Storage {
        // Shows how to create a custom struct in PublicMutable
        // docs:start:storage-leader-declaration
        leader: PublicMutable<Leader>,
        // docs:end:storage-leader-declaration
        // docs:start:storage-private-mutable-declaration
        legendary_card: PrivateMutable<CardNote>,
        // docs:end:storage-private-mutable-declaration
        // just used for docs example to show how to create a private mutable map. 
        profiles: Map<AztecAddress, PrivateMutable<CardNote>>,
        // docs:start:storage-set-declaration
        set: PrivateSet<CardNote>,
        // docs:end:storage-set-declaration
        // docs:start:storage-private-immutable-declaration
        private_immutable: PrivateImmutable<CardNote>,
        // docs:end:storage-private-immutable-declaration
        // docs:start:storage-shared-immutable-declaration
        shared_immutable: SharedImmutable<Leader>,
        // docs:end:storage-shared-immutable-declaration
        // docs:start:storage-minters-declaration
        minters: Map<AztecAddress, PublicMutable<bool>>,
        // docs:end:storage-minters-declaration
        // docs:start:storage-public-immutable-declaration
        public_immutable: PublicImmutable<Leader>,
        // docs:end:storage-public-immutable-declaration
    }

    // Note: The following is no longer necessary to implement manually as our macros do this for us. It is left here
    // for documentation purposes only.
    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                // docs:start:storage-leader-init
                leader: PublicMutable::new(context, 1),
                // docs:end:storage-leader-init
                // docs:start:start_vars_private_mutable
                legendary_card: PrivateMutable::new(context, 3),
                // docs:end:start_vars_private_mutable
                // just used for docs example (not for game play):
                // docs:start:state_vars-MapPrivateMutable
                profiles: Map::new(
                    context,
                    4,
                    |context, slot| {
                        PrivateMutable::new(context, slot)
                    }
                ),
                // docs:end:state_vars-MapPrivateMutable
                // docs:start:storage-set-init
                set: PrivateSet::new(context, 5),
                // docs:end:storage-set-init
                private_immutable: PrivateImmutable::new(context, 6),
                // docs:start:storage-shared-immutable
                shared_immutable: SharedImmutable::new(context, 7),
                // docs:end:storage-shared-immutable
                // docs:start:storage-minters-init
                minters: Map::new(
                    context,
                    8,
                    |context, slot| {
                        PublicMutable::new(context, slot)
                    }
                ),
                // docs:end:storage-minters-init
                // docs:start:storage-public-immutable
                public_immutable: PublicImmutable::new(context, 9)// docs:end:storage-public-immutable
            }
        }
    }

    #[aztec(public)]
    fn initialize_shared_immutable(points: u8) {
        let mut new_leader = Leader { account: context.msg_sender(), points };
        storage.shared_immutable.initialize(new_leader);
    }

    #[aztec(private)]
    fn match_shared_immutable(account: AztecAddress, points: u8) {
        let expected = Leader { account, points };
        let read = storage.shared_immutable.read_private();

        assert(read.account == expected.account, "Invalid account");
        assert(read.points == expected.points, "Invalid points");
    }

    #[aztec(private)]
    fn get_shared_immutable_constrained_private_indirect() -> pub Leader {
        // This is a private function that calls another private function
        // and returns the response.
        // Used to test that we can retrieve values through calls and 
        // correctly return them in the simulation
        let ret = context.call_private_function_no_args(
            context.this_address(),
            FunctionSelector::from_signature("get_shared_immutable_constrained_private()")
        );
        Leader::deserialize([ret[0], ret[1]])
    }

    #[aztec(public)]
    fn get_shared_immutable_constrained_public_indirect() -> pub Leader {
        // This is a public function that calls another public function
        // and returns the response.
        // Used to test that we can retrieve values through calls and 
        // correctly return them in the simulation
        let ret = context.call_public_function_no_args(
            context.this_address(),
            FunctionSelector::from_signature("get_shared_immutable_constrained_public()")
        );
        Leader::deserialize([ret[0], ret[1]])
    }

    #[aztec(public)]
    fn get_shared_immutable_constrained_public() -> pub Leader {
        storage.shared_immutable.read_public()
    }

    #[aztec(private)]
    fn get_shared_immutable_constrained_private() -> pub Leader {
        storage.shared_immutable.read_private()
    }

    unconstrained fn get_shared_immutable() -> pub Leader {
        storage.shared_immutable.read_public()
    }

    #[aztec(public)]
    fn initialize_public_immutable(points: u8) {
        // docs:start:initialize_public_immutable
        let mut new_leader = Leader { account: context.msg_sender(), points };
        storage.public_immutable.initialize(new_leader);
        // docs:end:initialize_public_immutable
    }

    unconstrained fn get_public_immutable() -> pub Leader {
        // docs:start:read_public_immutable
        storage.public_immutable.read()
        // docs:end:read_public_immutable
    }

    // docs:start:initialize-private-mutable
    #[aztec(private)]
    fn initialize_private_immutable(randomness: Field, points: u8) {
        let mut new_card = CardNote::new(points, randomness, context.msg_sender());
        storage.private_immutable.initialize(&mut new_card, true);
    }
    // docs:end:initialize-private-mutable

    #[aztec(private)]
    // msg_sender() is 0 at deploy time. So created another function
    fn initialize_private(randomness: Field, points: u8) {
        let mut legendary_card = CardNote::new(points, randomness, context.msg_sender());
        // create and broadcast note
        storage.legendary_card.initialize(&mut legendary_card, true);
    }

    #[aztec(private)]
    fn insert_notes(amounts: [u8; 10]) {
        for i in 0..amounts.len() {
            let mut note = CardNote::new(amounts[i], 1, context.msg_sender());
            storage.set.insert(&mut note, true);
        }
    }

    #[aztec(private)]
    fn insert_note(amount: u8, randomness: Field) {
        let mut note = CardNote::new(amount, randomness, context.msg_sender());
        storage.set.insert(&mut note, true);
    }

    // docs:start:state_vars-NoteGetterOptionsComparatorExampleNoir
    unconstrained fn read_note(amount: Field, comparator: u8) -> pub [Option<CardNote>; 10] {
        let options = NoteViewerOptions::new().select(
            CardNote::properties().points,
            amount,
            Option::some(comparator)
        );
        let notes = storage.set.view_notes(options);

        notes
    }
    // docs:end:state_vars-NoteGetterOptionsComparatorExampleNoir

    #[aztec(private)]
    fn update_legendary_card(randomness: Field, points: u8) {
        let mut new_card = CardNote::new(points, randomness, context.msg_sender());
        storage.legendary_card.replace(&mut new_card, true);

        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("update_leader((Field),u8)"),
            [context.msg_sender().to_field(), points as Field]
        );
    }

    #[aztec(private)]
    fn increase_legendary_points() {
        // Ensure `points` > current value
        // Also serves as a e2e test that you can `get_note()` and then `replace()`

        // docs:start:state_vars-PrivateMutableGet
        let card = storage.legendary_card.get_note(false);
        // docs:end:state_vars-PrivateMutableGet

        let points = card.points + 1;

        let mut new_card = CardNote::new(points, card.randomness, context.msg_sender());
        // docs:start:state_vars-PrivateMutableReplace
        storage.legendary_card.replace(&mut new_card, true);
        // docs:end:state_vars-PrivateMutableReplace

        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("update_leader((Field),u8)"),
            [context.msg_sender().to_field(), points as Field]
        );
    }

    #[aztec(private)]
    fn spend_private_authwit(inner_hash: Field) -> Field {
        1
    }

    #[aztec(public)]
    fn spend_public_authwit(inner_hash: Field) -> Field {
        1
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn update_leader(account: AztecAddress, points: u8) {
        let new_leader = Leader { account, points };
        storage.leader.write(new_leader);
    }

    unconstrained fn get_leader() -> pub Leader {
        storage.leader.read()
    }

    unconstrained fn get_legendary_card() -> pub CardNote {
        storage.legendary_card.view_note()
    }

    // docs:start:private_mutable_is_initialized
    unconstrained fn is_legendary_initialized() -> pub bool {
        storage.legendary_card.is_initialized()
    }
    // docs:end:private_mutable_is_initialized

    // docs:start:get_note-private-immutable
    unconstrained fn get_imm_card() -> pub CardNote {
        storage.private_immutable.get_note()
    }
    // docs:end:get_note-private-immutable

    unconstrained fn view_imm_card() -> pub CardNote {
        storage.private_immutable.view_note()
    }

    unconstrained fn is_priv_imm_initialized() -> pub bool {
        storage.private_immutable.is_initialized()
    }

    /// Macro equivalence section
    use dep::aztec::hash::hash_args;

    use dep::aztec::protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs;
    use dep::aztec::context::inputs::PrivateContextInputs;

    // docs:start:simple_macro_example
    #[aztec(private)]
    fn simple_macro_example(a: Field, b: Field) -> Field {
        a + b
    }
    // docs:end:simple_macro_example

    // docs:start:simple_macro_example_expanded
    fn simple_macro_example_expanded(
        // ************************************************************
        // The private context inputs are made available to the circuit by the kernel
        // docs:start:context-example-inputs
        inputs: PrivateContextInputs,
        // docs:end:context-example-inputs
        // ************************************************************

        // Our original inputs!
        a: Field,
        b: Field // The actual return type of our circuit is the PrivateCircuitPublicInputs struct, this will be the 
        // input to our kernel! 
        // docs:start:context-example-return
    ) -> distinct pub PrivateCircuitPublicInputs {
        // docs:end:context-example-return
        // ************************************************************
        // The hasher is a structure used to generate a hash of the circuits inputs. 
        // docs:start:context-example-hasher
        let mut args_hasher = dep::aztec::hash::ArgsHasher::new();
        args_hasher.add(a);
        args_hasher.add(b);
        // docs:end:context-example-hasher

        // The context object is created with the inputs and the hash of the inputs
        // docs:start:context-example-context
        let mut context = PrivateContext::new(inputs, args_hasher.hash());
        // docs:end:context-example-context

        // docs:start:storage-example-context
        let mut storage = Storage::init(Context::private(&mut context));
        // docs:end:storage-example-context
        // ************************************************************

        // Our actual program
        let result = a + b;

        // ************************************************************
        // Return values are pushed into the context
        // docs:start:context-example-context-return
        context.return_values.push(result);
        // docs:end:context-example-context-return

        // The context is returned to be consumed by the kernel circuit!
        // docs:start:context-example-finish
        context.finish()
        // docs:end:context-example-finish
        // ************************************************************
    }
    // docs:end:simple_macro_example_expanded
}
