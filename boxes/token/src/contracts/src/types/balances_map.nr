use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::std::option::Option;
use crate::types::balance_set::BalanceSet;
use dep::aztec::hash::pedersen_hash;
use dep::aztec::protocol_types::address::AztecAddress;

use crate::types::token_note::{TokenNote, TOKEN_NOTE_LEN, TokenNoteMethods};
use dep::aztec::state_vars::{map::Map, set::Set};

struct BalancesMap {
    store: Map<AztecAddress, Set<TokenNote, TOKEN_NOTE_LEN>>,
}

impl BalancesMap {
    pub fn new(
        context: Context,
        storage_slot: Field,
    ) -> Self {
        let store = Map::new(context, storage_slot, |context, storage_slot| {
            Set {
                context,
                storage_slot,
                note_interface: TokenNoteMethods,
            }
        });
        Self {
            store,
        }
    }

    pub fn at(self, owner: AztecAddress) -> BalanceSet {
        let set = self.store.at(owner);
        BalanceSet::new(set, owner)
    }
}
