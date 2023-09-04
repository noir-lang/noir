mod storage;

contract ExamplePublicStateIncrement {
    use dep::aztec::abi;
    use dep::aztec::abi::Hasher;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::{
        PrivateContext,
        PublicContext,
    };
    use dep::aztec::types::point::Point;
    use crate::storage::Storage;
    use dep::aztec::state_vars::{
        type_serialisation::u32_serialisation::{
            U32SerialisationMethods,
            U32_SERIALISED_LEN,
        },
    };

    // call initialise_a();
    #[aztec(private)]
    fn constructor(
    ) {
        let initialise_a_function_selector: Field = 1234;
        let _return_values = context.call_public_function_no_args(context.this_address(), initialise_a_function_selector);
    }

    // a = 100;
    #[aztec(public)]
    internal fn initialise_a() {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        storage.a.write(100);
    }

    // a += b;
    #[aztec(public)]
    fn increment_a(
        b: Field,
    ) {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        let mut a = storage.a.read();
        a += b;
        storage.a.write(a);
    }
}
