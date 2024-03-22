// An entrypoint contract that allows everything to go through. Only used for testing
// Pair this with SignerlessWallet to perform multiple actions before any account contracts are deployed (and without authentication)
contract MultiCallEntrypoint {
    use dep::std;

    use dep::aztec::prelude::AztecAddress;
    use dep::authwit::entrypoint::app::AppPayload;

    #[aztec(private)]
    fn entrypoint(app_payload: pub AppPayload) {
        app_payload.execute_calls(&mut context);
    }
}
