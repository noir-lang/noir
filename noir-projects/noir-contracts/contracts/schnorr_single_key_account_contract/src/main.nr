mod util;
mod auth_oracle;

contract SchnorrSingleKeyAccount {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, PrivateContext};

    use dep::authwit::{entrypoint::{app::AppPayload, fee::FeePayload}, account::AccountActions};

    use crate::{util::recover_address, auth_oracle::get_auth_witness};

    global ACCOUNT_ACTIONS_STORAGE_SLOT = 1;

    // Note: If you globally change the entrypoint signature don't forget to update default_entrypoint.ts
    #[aztec(private)]
    fn entrypoint(app_payload: pub AppPayload, fee_payload: pub FeePayload) {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.entrypoint(app_payload, fee_payload);
    }

    #[aztec(private)]
    fn spend_private_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_private_authwit(inner_hash)
    }

    #[aztec(public)]
    fn spend_public_authwit(inner_hash: Field) -> Field {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.spend_public_authwit(inner_hash)
    }

    #[aztec(private)]
    #[aztec(internal)]
    fn cancel_authwit(outer_hash: Field) {
        context.push_new_nullifier(outer_hash, 0);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn approve_public_authwit(outer_hash: Field) {
        let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
        actions.approve_public_authwit(outer_hash)
    }

    #[contract_library_method]
    fn is_valid_impl(context: &mut PrivateContext, outer_hash: Field) -> bool {
        let witness = get_auth_witness(outer_hash);
        assert(recover_address(outer_hash, witness).eq(context.this_address()));
        true
    }
}
