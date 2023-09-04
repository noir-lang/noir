use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::{
    immutable_singleton::ImmutableSingleton, map::Map, public_state::PublicState, set::Set,
    singleton::Singleton,
};
use dep::std::option::Option;
// docs:start:state_vars-PublicStateBoolImport
use dep::aztec::types::type_serialisation::bool_serialisation::{
    BoolSerialisationMethods, BOOL_SERIALISED_LEN,
};
// docs:end:state_vars-PublicStateBoolImport

use crate::types::{
    card_note::{CardNote, CardNoteMethods, CARD_NOTE_LEN},
    profile_note::{ProfileNote, ProfileNoteMethods, PROFILE_NOTE_LEN},
    queen::{Queen, QueenSerialisationMethods, QUEEN_SERIALISED_LEN},
    rules_note::{RulesNote, RulesNoteMethods, RULES_NOTE_LEN},
};

// docs:start:storage-declaration
struct Storage {
    locked: PublicState<bool, BOOL_SERIALISED_LEN>,
    queen: PublicState<Queen, QUEEN_SERIALISED_LEN>,
    game_rules: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>,
    legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
    cards: Set<CardNote, CARD_NOTE_LEN>,
    profiles: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>,
}

// docs:start:state_vars-PublicState
// docs:start:state_vars-PublicStateCustomStruct
// docs:start:state_vars-Singleton
// docs:start:state_vars-ImmutableSingleton
// docs:start:state_vars-Set
// docs:start:state_vars-MapSingleton
impl Storage {
    fn init(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
    ) -> Self {
        Storage {
            // highlight-next-line:state_vars-PublicState
            locked: PublicState::new(private_context, public_context, 1, BoolSerialisationMethods),
            // highlight-next-line:state_vars-PublicStateCustomStruct
            queen: PublicState::new(
                private_context,
                public_context,
                2,
                QueenSerialisationMethods,
            ),
            // highlight-next-line:state_vars-ImmutableSingleton
            game_rules: ImmutableSingleton::new(private_context, 3, RulesNoteMethods),
            // highlight-next-line:state_vars-Singleton
            legendary_card: Singleton::new(private_context, public_context, 4, CardNoteMethods),
            // highlight-next-line:state_vars-Set
            cards: Set::new(private_context, public_context, 5, CardNoteMethods),
            // highlight-next-line:state_vars-MapSingleton
            profiles: Map::new(
                private_context,
                public_context,
                6,
                |private_context, public_context, slot| {
                    Singleton::new(private_context, public_context, slot, ProfileNoteMethods)
                },
            ),
        }
    }
}
// docs:end:state_vars-PublicState
// docs:end:state_vars-PublicStateCustomStruct
// docs:end:state_vars-Singleton
// docs:end:state_vars-ImmutableSingleton
// docs:end:state_vars-Set
// docs:end:state_vars-MapSingleton
// docs:end:storage-declaration
