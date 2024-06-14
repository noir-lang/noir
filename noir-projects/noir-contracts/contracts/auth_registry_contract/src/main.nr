contract AuthRegistry {
    use dep::aztec::{state_vars::{PublicMutable, Map}, protocol_types::address::AztecAddress};
    use dep::authwit::auth::{IS_VALID_SELECTOR, compute_outer_authwit_hash, assert_current_call_valid_authwit};

    #[aztec(storage)]
    struct Storage {
      reject_all: Map<AztecAddress, PublicMutable<bool>>,
      // on_behalf_of => authwit hash => authorized
      approved_actions: Map<AztecAddress, Map<Field, PublicMutable<bool>>>,
    }

    /**
     * Updates the `authorized` value for `msg_sender` for `message_hash`.
     *
     * @param message_hash The message hash being authorized
     * @param authorize True if the caller is authorized to perform the message hash, false otherwise
     */
    #[aztec(public)]
    fn set_authorized(message_hash: Field, authorize: bool) {
        storage.approved_actions.at(context.msg_sender()).at(message_hash).write(authorize);
    }

    /**
     * Updates the `reject_all` value for `msg_sender`.
     * 
     * When `reject_all` is `true` any `consume` on `msg_sender` will revert.
     * 
     * @param reject True if all actions should be rejected, false otherwise
     */
    #[aztec(public)]
    fn set_reject_all(reject: bool) {
        storage.reject_all.at(context.msg_sender()).write(reject);
    }

    /**
     * Consumes an `inner_hash` on behalf of `on_behalf_of` if the caller is authorized to do so.
     * 
     * Will revert even if the caller is authorized if `reject_all` is set to true for `on_behalf_of`.
     * This is to support "mass-revoke".
     *
     * @param on_behalf_of The address on whose behalf the action is being consumed
     * @param inner_hash The inner_hash of the authwit
     * @return `IS_VALID_SELECTOR` if the action was consumed, revert otherwise
     */
    #[aztec(public)]
    fn consume(on_behalf_of: AztecAddress, inner_hash: Field) -> Field {
        assert_eq(false, storage.reject_all.at(on_behalf_of).read(), "rejecting all");

        let message_hash = compute_outer_authwit_hash(
            context.msg_sender(),
            context.chain_id(),
            context.version(),
            inner_hash
        );

        let authorized = storage.approved_actions.at(on_behalf_of).at(message_hash).read();

        assert_eq(true, authorized, "unauthorized");
        storage.approved_actions.at(on_behalf_of).at(message_hash).write(false);

        IS_VALID_SELECTOR
    }

    /**
     * Updates a public authwit using a private authwit
     * 
     * Useful for the case where you want someone else to insert a public authwit for you.
     * For example, if Alice wants Bob to insert an authwit in public, such that they can execute
     * a trade, Alice can create a private authwit, and Bob can call this function with it.
     *
     * @param approver The address of the approver (Alice in the example)
     * @param message_hash The message hash to authorize
     * @param authorize True if the message hash should be authorized, false otherwise
     */
    #[aztec(private)]
    fn set_authorized_private(approver: AztecAddress, message_hash: Field, authorize: bool) {
        assert_current_call_valid_authwit(&mut context, approver);
        AuthRegistry::at(context.this_address())._set_authorized(approver, message_hash, authorize).enqueue(&mut context);
    }

    /**
     * Internal function to update the `authorized` value for `approver` for `messageHash`.
     * Used along with `set_authorized_private` to update the public authwit.
     * 
     * @param approver The address of the approver
     * @param message_hash The message hash being authorized
     * @param authorize True if the caller is authorized to perform the message hash, false otherwise
     */
    #[aztec(public)]
    #[aztec(internal)]
    fn _set_authorized(approver: AztecAddress, message_hash: Field, authorize: bool) {
        storage.approved_actions.at(approver).at(message_hash).write(authorize);
    }

    /**
     * Fetches the `reject_all` value for `on_behalf_of`.
     * 
     * @param on_behalf_of The address to check
     * @return True if all actions are rejected, false otherwise
     */
    #[aztec(public)]
    #[aztec(view)]
    fn is_reject_all(on_behalf_of: AztecAddress) -> bool {
        storage.reject_all.at(on_behalf_of).read()
    }

    /**
     * Fetches the `authorized` value for `on_behalf_of` for `message_hash`.
     * 
     * @param on_behalf_of The address on whose behalf the action is being consumed
     * @param message_hash The message hash to check
     * @return True if the caller is authorized to perform the action, false otherwise
     */
    #[aztec(public)]
    #[aztec(view)]
    fn is_consumable(on_behalf_of: AztecAddress, message_hash: Field) -> bool {
        storage.approved_actions.at(on_behalf_of).at(message_hash).read()
    }

    unconstrained fn unconstrained_is_consumable(on_behalf_of: AztecAddress, message_hash: Field) -> pub bool {
        storage.approved_actions.at(on_behalf_of).at(message_hash).read()
    }
}
