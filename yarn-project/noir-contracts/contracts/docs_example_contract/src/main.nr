mod actions;
mod options;
mod types;

contract DocsExample {
    use dep::aztec::protocol_types::{
        address::AztecAddress,
    };
    use dep::aztec::{
        context::{PrivateContext, Context},
        state_vars::{
            map::Map,
            singleton::Singleton,
        },
    };
    use crate::actions;
    use crate::options::create_account_card_getter_options;
    use crate::types::{
        card_note::{CardNote, CardNoteMethods, CARD_NOTE_LEN},
    };

    struct Storage {
        // docs:start:storage-singleton-declaration
        legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
        // docs:end:storage-singleton-declaration
        // docs:start:storage-map-singleton-declaration
        profiles: Map<AztecAddress, Singleton<CardNote, CARD_NOTE_LEN>>,
        // docs:end:storage-map-singleton-declaration
    }

    // docs:start:state_vars-MapSingleton
    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                // docs:start:start_vars_singleton
                legendary_card: Singleton::new(context, 1, CardNoteMethods),
                // docs:end:start_vars_singleton
                // highlight-next-line:state_vars-MapSingleton
                profiles: Map::new(
                    context,
                    2,
                    |context, slot| {
                        Singleton::new(context, slot, CardNoteMethods)
                    },
                ),
            }
        }
    }
    // docs:end:state_vars-MapSingleton

    #[aztec(private)]
    fn constructor(legendary_card_secret: Field) {
        let mut legendary_card = CardNote::new(0, legendary_card_secret, AztecAddress::zero());
        actions::init_legendary_card(storage.legendary_card, &mut legendary_card);
    }

    #[aztec(private)]
    fn update_legendary_card(new_points: u8, new_secret: Field) {
        let owner = inputs.call_context.msg_sender;
        let mut updated_card = CardNote::new(new_points, new_secret, owner);
        actions::update_legendary_card(storage.legendary_card, &mut updated_card);
    }

    unconstrained fn get_legendary_card() -> pub CardNote {
        actions::get_legendary_card(storage.legendary_card)
    }

    /// Macro equivalence section
    use dep::aztec::abi;
    use dep::aztec::abi::Hasher;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PrivateCircuitPublicInputs;

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
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // docs:end:context-example-return
        // ************************************************************
        // The hasher is a structure used to generate a hash of the circuits inputs. 
        // docs:start:context-example-hasher
        let mut hasher = Hasher::new();
        hasher.add(a);
        hasher.add(b);
        // docs:end:context-example-hasher

        // The context object is created with the inputs and the hash of the inputs
        // docs:start:context-example-context
        let mut context = PrivateContext::new(inputs, hasher.hash());
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

    // Cross chain messaging section
    // Demonstrates a cross chain message 
    // docs:start:l1_to_l2_cross_chain_message 
    #[aztec(private)]
    fn send_to_l1() {}
    // docs:end:l1_to_l2_cross_chain_message  

    // TODO: remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; 0]
    ) -> pub [Field; 4] {
        [0, 0, 0, 0]
    }
}
