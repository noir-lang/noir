mod util;
mod auth_oracle;

contract SchnorrAuthWitnessAccount {
    use dep::std::hash::pedersen_with_separator;
    use dep::aztec::entrypoint::EntrypointPayload;
    use dep::aztec::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;
    use crate::util::recover_address;
    use crate::auth_oracle::get_auth_witness;

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn entrypoint(
        payload: pub EntrypointPayload,
    ) {
        let message_hash: Field = pedersen_with_separator(
          payload.serialize(), 
          GENERATOR_INDEX__SIGNATURE_PAYLOAD
        )[0];
        _inner_is_valid(message_hash, context.this_address());
        payload.execute_calls(&mut context);
    }

    #[aztec(private)]
    fn is_valid(
        message_hash: Field
    ) -> Field {
        _inner_is_valid(message_hash, context.this_address());
        0xe86ab4ff
    }

    fn _inner_is_valid(
        message_hash: Field,
        address: Field,
    ) {
        let witness = get_auth_witness(message_hash);
        assert(recover_address(message_hash, witness) == address);
    }
}