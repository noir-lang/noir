use crate::context::{PrivateContext, PublicContext};
use dep::std::option::Option;

struct Map<V> {
    private_context: Option<&mut PrivateContext>,
    public_context: Option<&mut PublicContext>,
    storage_slot: Field,
    state_var_constructor: fn(Option<&mut PrivateContext>, Option<&mut PublicContext>, Field) -> V,
}

impl<V> Map<V> {
    fn new(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
        storage_slot: Field,
        state_var_constructor: fn(
            Option<&mut PrivateContext>,
            Option<&mut PublicContext>,
            Field,
        ) -> V,
    ) -> Map<V> {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Map {
            private_context,
            public_context,
            storage_slot,
            state_var_constructor,
        }
    }

    fn at(self, key: Field) -> V {
        // TODO(#1204): use a generator index for the storage slot
        let derived_storage_slot = dep::std::hash::pedersen([self.storage_slot, key])[0];

        let state_var_constructor = self.state_var_constructor;
        state_var_constructor(
            self.private_context,
            self.public_context,
            derived_storage_slot,
        )
    }
}
