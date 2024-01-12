use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::std::option::Option;
use crate::types::balance_set::BalanceSet;
use dep::aztec::hash::pedersen_hash;
use dep::aztec::protocol_types::address::AztecAddress;

struct BalancesMap {
    context: Context,
    storage_slot: Field,
}

impl BalancesMap {
    pub fn new(
        context: Context,
        storage_slot: Field,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self {
            context,
            storage_slot,
        }
    }

    pub fn at(self, owner: AztecAddress) -> BalanceSet {
        let derived_storage_slot = pedersen_hash([self.storage_slot, owner.to_field()], 0);
        BalanceSet::new(self.context, owner, derived_storage_slot)
    }
}
