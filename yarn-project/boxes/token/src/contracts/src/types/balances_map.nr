use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::aztec::types::address::AztecAddress;
use dep::std::option::Option;
use crate::types::balance_set::BalanceSet;

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
        let derived_storage_slot = dep::std::hash::pedersen([self.storage_slot, owner.address])[0];
        BalanceSet::new(self.context, owner, derived_storage_slot)
    }
}
