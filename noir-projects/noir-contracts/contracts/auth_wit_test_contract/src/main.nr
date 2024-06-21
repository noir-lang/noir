contract AuthWitTest {
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::authwit::auth::{assert_inner_hash_valid_authwit, assert_inner_hash_valid_authwit_public};

    #[aztec(private)]
    fn consume(on_behalf_of: AztecAddress, inner_hash: Field) {
        assert_inner_hash_valid_authwit(&mut context, on_behalf_of, inner_hash);
    }

    #[aztec(public)]
    fn consume_public(on_behalf_of: AztecAddress, inner_hash: Field) {
        assert_inner_hash_valid_authwit_public(&mut context, on_behalf_of, inner_hash);
    }
}
