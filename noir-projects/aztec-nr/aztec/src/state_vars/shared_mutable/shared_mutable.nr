use dep::protocol_types::{
    hash::{pedersen_hash, poseidon2_hash}, header::Header, address::AztecAddress,
    traits::{FromField, ToField}
};

use crate::context::{PrivateContext, PublicContext, UnconstrainedContext};
use crate::state_vars::{
    storage::Storage,
    shared_mutable::{scheduled_value_change::ScheduledValueChange, scheduled_delay_change::ScheduledDelayChange}
};
use crate::oracle::storage::storage_read;
use dep::std::unsafe::zeroed;

mod test;

struct SharedMutable<T, let INITIAL_DELAY: u32, Context> {
    context: Context,
    storage_slot: Field,
}

// This will make the Aztec macros require that T implements the Serialize<N> trait, and allocate N storage slots to
// this state variable. This is incorrect, since what we actually store is:
//  - a ScheduledValueChange<T>, which requires 1 + 2 * M storage slots, where M is the serialization length of T
//  - a ScheduledDelayChange, which requires another storage slot
//
// TODO https://github.com/AztecProtocol/aztec-packages/issues/5736: change the storage allocation scheme so that we
// can actually use it here
impl<T, INITIAL_DELAY, Context> Storage<T> for SharedMutable<T, INITIAL_DELAY, Context> {}

// TODO: extract into a utils module once  we can do arithmetic on generics, i.e. https://github.com/noir-lang/noir/issues/4784
fn concat_arrays<N, M, O>(arr_n: [Field; N], arr_m: [Field; M]) -> [Field; O] {
    assert_eq(N + M, O);
    let mut out: [Field; O] = [0; O];
    for i in 0..N {
        out[i] = arr_n[i];
    }
    for i in 0..M {
        out[N+i] = arr_m[i];
    }
    out
}

// SharedMutable<T> stores a value of type T that is:
//  - publicly known (i.e. unencrypted)
//  - mutable in public
//  - readable in private with no contention (i.e. multiple parties can all read the same value without blocking one
//    another nor needing to coordinate)
// This is famously a hard problem to solve. SharedMutable makes it work by introducing a delay to public mutation:
// the value is not changed immediately but rather a value change is scheduled to happen in the future after some delay
// measured in blocks. Reads in private are only valid as long as they are included in a block not too far into the
// future, so that they can guarantee the value will not have possibly changed by then (because of the delay).
// The delay for changing a value is initially equal to INITIAL_DELAY, but can be changed by calling
// `schedule_delay_change`.
impl<T, INITIAL_DELAY, Context> SharedMutable<T, INITIAL_DELAY, Context> where T: ToField + FromField + Eq {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }

    fn hash_scheduled_data(
        value_change: ScheduledValueChange<T>,
        delay_change: ScheduledDelayChange<INITIAL_DELAY>
    ) -> Field {
        // TODO(#5491 and https://github.com/noir-lang/noir/issues/4784): update this so that we don't need to rely on
        // ScheduledValueChange serializing to 3 and ScheduledDelayChange serializing to 1
        let concatenated: [Field; 4] = concat_arrays(value_change.serialize(), delay_change.serialize());
        poseidon2_hash(concatenated)
    }

    // Since we can't rely on the native storage allocation scheme, we hash the storage slot to get a unique location in
    // which we can safely store as much data as we need.
    // See https://github.com/AztecProtocol/aztec-packages/issues/5492 and
    // https://github.com/AztecProtocol/aztec-packages/issues/5736
    // We store three things in public storage:
    //  - a ScheduledValueChange
    //  - a ScheduledDelaChange
    //  - the hash of both of these (via `hash_scheduled_data`)
    fn get_value_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 0], 0)
    }

    fn get_delay_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 1], 0)
    }

    fn get_hash_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 2], 0)
    }

    // It may seem odd that we take a header and address instead of reading from e.g. a PrivateContext, but this lets us
    // reuse this function in SharedMutablePrivateGetter.
    fn historical_read_from_public_storage(
        self,
        header: Header,
        address: AztecAddress
    ) -> (ScheduledValueChange<T>, ScheduledDelayChange<INITIAL_DELAY>, u32) {
        let historical_block_number = header.global_variables.block_number as u32;

        // We could simply produce historical inclusion proofs for both the ScheduledValueChange and
        // ScheduledDelayChange, but that'd require one full sibling path per storage slot (since due to kernel siloing
        // the storage is not contiguous), and in the best case in which T is a single field that'd be 4 slots.
        // Instead, we get an oracle to provide us the correct values for both the value and delay changes, and instead
        // prove inclusion of their hash, which is both a much smaller proof (a single slot), and also independent of
        // the size of T.
        let (value_change_hint, delay_change_hint) = get_public_storage_hints(address, self.storage_slot, historical_block_number);

        // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
        let hash = header.public_storage_historical_read(self.get_hash_storage_slot(), address);

        // @todo This is written strangely to bypass a formatting issue with the if that is breaking ci.
        let (a, b, c) = if hash != 0 {
            let a = SharedMutable::hash_scheduled_data(value_change_hint, delay_change_hint);
            (a, value_change_hint, delay_change_hint)
        } else {
            // The hash slot can only hold a zero if it is uninitialized, meaning no value or delay change was ever
            // scheduled. Therefore, the hints must then correspond to uninitialized scheduled changes.
            let b = ScheduledValueChange::deserialize(zeroed());
            let c = ScheduledDelayChange::deserialize(zeroed());
            (hash, b, c)
        };

        assert_eq(hash, a, "Hint values do not match hash");
        assert_eq(value_change_hint, b, "Non-zero value change for zero hash");
        assert_eq(delay_change_hint, c, "Non-zero delay change for zero hash");

        (value_change_hint, delay_change_hint, historical_block_number)
    }
}

impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY, &mut PublicContext> where T: ToField + FromField + Eq {
    pub fn schedule_value_change(self, new_value: T) {
        let mut value_change = self.read_value_change();
        let delay_change = self.read_delay_change();

        let block_number = self.context.block_number() as u32;
        let current_delay = delay_change.get_current(block_number);

        // TODO: make this configurable
        // https://github.com/AztecProtocol/aztec-packages/issues/5501
        let block_of_change = block_number + current_delay;
        value_change.schedule_change(new_value, block_number, current_delay, block_of_change);

        self.write(value_change, delay_change);
    }

    pub fn schedule_delay_change(self, new_delay: u32) {
        let mut delay_change = self.read_delay_change();

        let block_number = self.context.block_number() as u32;

        delay_change.schedule_change(new_delay, block_number);

        self.write(self.read_value_change(), delay_change);
    }

    pub fn get_current_value_in_public(self) -> T {
        let block_number = self.context.block_number() as u32;
        self.read_value_change().get_current_at(block_number)
    }

    pub fn get_current_delay_in_public(self) -> u32 {
        let block_number = self.context.block_number() as u32;
        self.read_delay_change().get_current(block_number)
    }

    pub fn get_scheduled_value_in_public(self) -> (T, u32) {
        self.read_value_change().get_scheduled()
    }

    pub fn get_scheduled_delay_in_public(self) -> (u32, u32) {
        self.read_delay_change().get_scheduled()
    }

    fn read_value_change(self) -> ScheduledValueChange<T> {
        self.context.storage_read(self.get_value_change_storage_slot())
    }

    fn read_delay_change(self) -> ScheduledDelayChange<INITIAL_DELAY> {
        self.context.storage_read(self.get_delay_change_storage_slot())
    }

    fn write(
        self,
        value_change: ScheduledValueChange<T>,
        delay_change: ScheduledDelayChange<INITIAL_DELAY>
    ) {
        // Whenever we write to public storage, we write both the value change and delay change as well as the hash of
        // them both. This guarantees that the hash is always kept up to date.
        // While this makes for more costly writes, it also makes private proofs much simpler because they only need to
        // produce a historical proof for the hash, which results in a single inclusion proof (as opposed to 4 in the
        // best case scenario in which T is a single field). Private shared mutable reads are assumed to be much more
        // frequent than public writes, so this tradeoff makes sense.
        self.context.storage_write(self.get_value_change_storage_slot(), value_change);
        self.context.storage_write(self.get_delay_change_storage_slot(), delay_change);
        self.context.storage_write(
            self.get_hash_storage_slot(),
            SharedMutable::hash_scheduled_data(value_change, delay_change)
        );
    }
}

impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY, &mut PrivateContext> where T: ToField + FromField + Eq {
    pub fn get_current_value_in_private(self) -> T {
        // When reading the current value in private we construct a historical state proof for the public value.
        // However, since this value might change, we must constrain the maximum transaction block number as this proof
        // will only be valid for however many blocks we can ensure the value will not change, which will depend on the
        // current delay and any scheduled delay changes.

        let (value_change, delay_change, historical_block_number) = self.historical_read_from_public_storage(self.context.get_header(), self.context.this_address());

        // We use the effective minimum delay as opposed to the current delay at the historical block as this one also
        // takes into consideration any scheduled delay changes.
        // For example, consider a scenario in which at block 200 the current delay was 50. We may naively think that
        // the earliest we could change the value would be at block 251 by scheduling immediately after the historical
        // block, i.e. at block 201. But if there was a delay change scheduled for block 210 to reduce the delay to 20
        // blocks, then if a value change was scheduled at block 210 it would go into effect at block 230, which is
        // earlier than what we'd expect if we only considered the current delay.
        let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
        let block_horizon = value_change.get_block_horizon(historical_block_number, effective_minimum_delay);

        // We prevent this transaction from being included in any block after the block horizon, ensuring that the
        // historical public value matches the current one, since it can only change after the horizon.
        self.context.set_tx_max_block_number(block_horizon);
        value_change.get_current_at(historical_block_number)
    }
}

impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY, UnconstrainedContext> where T: ToField + FromField + Eq {
    unconstrained pub fn get_current_value_in_unconstrained(self) -> T {
        let block_number = self.context.block_number() as u32;
        self.read_value_change().get_current_at(block_number)
    }

    unconstrained fn read_value_change(self) -> ScheduledValueChange<T> {
        self.context.storage_read(self.get_value_change_storage_slot())
    }
}

unconstrained fn get_public_storage_hints<T, INITIAL_DELAY>(
    address: AztecAddress,
    storage_slot: Field,
    block_number: u32
) -> (ScheduledValueChange<T>, ScheduledDelayChange<INITIAL_DELAY>) where T: ToField + FromField + Eq {
    // This function cannot be part of the &mut PrivateContext impl because that'd mean that by passing `self` we'd also
    // be passing a mutable reference to an unconstrained function, which is not allowed. We therefore create a dummy
    // state variable here so that we can access the methods to compute storage slots. This will all be removed in the
    // future once we do proper storage slot allocation (#5492).
    let dummy: SharedMutable<T, INITIAL_DELAY, ()> = SharedMutable::new((), storage_slot);

    (
        storage_read(address, dummy.get_value_change_storage_slot(), block_number), storage_read(address, dummy.get_delay_change_storage_slot(), block_number)
    )
}
