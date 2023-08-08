mod storage;

contract ExamplePublicStateIncrement {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::Context;
    use dep::aztec::oracle::logs::emit_unencrypted_log;
    use dep::aztec::types::point::Point;
    use crate::storage::Storage;
    use dep::aztec::state_vars::{
        type_serialisation::u32_serialisation::{
            U32SerialisationMethods,
            U32_SERIALISED_LEN,
        },
    };

    // call initialise_a();
    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, 0);

        let initialise_a_function_selector: Field = 1234;
        let _return_values = context.call_public_function_no_args(context.this_address(), initialise_a_function_selector);
        
        context.finish()
    }

    // a = 100;
    open internal fn initialise_a(
        _inputs: PublicContextInputs,
    ) {
        let storage = Storage::init();
        storage.a.write(100);
    }

    // a += b;
    open fn increment_a(
        _inputs: PublicContextInputs,
        b: Field,
    ) {
        let storage = Storage::init();
        let mut a = storage.a.read();
        a += b;
        storage.a.write(a);
    }
}
