// A contract used along with `Parent` contract to test nested calls.
contract Child {
    use dep::std::option::Option;

    use dep::aztec::{
        abi::CallContext,
        context::{PrivateContext, PublicContext},
        oracle::{
            logs::emit_unencrypted_log,
            compute_selector::compute_selector,
        },
        state_vars::public_state::PublicState,
        types::type_serialisation::field_serialisation::{FieldSerialisationMethods, FIELD_SERIALISED_LEN},
    };

    struct Storage {
        current_value: PublicState<Field, FIELD_SERIALISED_LEN>,
    }

    impl Storage {
        fn init(
            private_context: Option<&mut PrivateContext>,
            public_context: Option<&mut PublicContext>,
        ) -> pub Self {
            Storage {
                current_value: PublicState::new(
                    private_context,
                    public_context,
                    1,
                    FieldSerialisationMethods,
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor()  {}

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values.
    #[aztec(private)]
    fn value(
        input: Field,
    ) -> Field {
        input + context.chain_id() + context.version()
    }

    fn check_sender(call_context: CallContext) {
        assert_eq(call_context.msg_sender, call_context.storage_contract_address, "Sender must be this contract");
    }

    // Returns a sum of the input and the chain id and version of the contract in private circuit public input's return_values. 
    // Can only be called from this contract.
    #[aztec(private)]
    fn valueInternal(
        input: Field,
    ) -> Field {
        check_sender(inputs.call_context);
        input + context.chain_id() + context.version()
    }

    // Returns base_value + chain_id + version + block_number + timestamp
    #[aztec(public)]
    fn pubGetValue(base_value: Field) -> Field {
        let returnValue = base_value + context.chain_id() + context.version() + context.block_number() + context.timestamp();

        returnValue
    }

    // Sets `current_value` to `new_value`
    #[aztec(public)]
    fn pubSetValue(new_value: Field) -> Field {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        storage.current_value.write(new_value);
        let _hash = emit_unencrypted_log(new_value);
        
        new_value
    }

    // Increments `current_value` by `new_value`
    #[aztec(public)]
    fn pubIncValue(new_value: Field) -> Field {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        let _hash = emit_unencrypted_log(new_value);

        new_value
    }

    // Increments `current_value` by `new_value`. Can only be called from this contract.
    #[aztec(public)] 
    fn pubIncValueInternal(new_value: Field) -> Field {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        check_sender(inputs.call_context);
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        let _hash = emit_unencrypted_log(new_value);
        
        new_value
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedFirst() {
        let pubSetValueSelector = compute_selector("pubSetValue(Field)");
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);

        let storage = Storage::init(Option::none(), Option::some(&mut context));
        storage.current_value.write(20);
        let _hash = emit_unencrypted_log(20);
    }

    #[aztec(public)]
    fn setValueTwiceWithNestedLast() {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        storage.current_value.write(20);
        let _hash = emit_unencrypted_log(20);

        let pubSetValueSelector = compute_selector("pubSetValue(Field)");
        let _ret = context.call_public_function(context.this_address(), pubSetValueSelector, [10]);
    }
}
