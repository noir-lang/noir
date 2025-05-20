use super::traits::{FromField, ToField};

pub(crate) use scheduled_delay_change::ScheduledDelayChange;
pub(crate) use scheduled_value_change::ScheduledValueChange;
pub(crate) use shared_mutable_values::SharedMutableValues;

pub(crate) mod scheduled_delay_change;
pub(crate) mod scheduled_value_change;
pub(crate) mod shared_mutable_values;
pub(crate) mod with_hash;

pub(crate) fn compute_shared_mutable_block_horizon<T, let INITIAL_DELAY: u32>(
    shared_mutable_values: SharedMutableValues<T, INITIAL_DELAY>,
    historical_block_number: u32,
) -> u32
where
    T: ToField + Eq + FromField,
{
    let effective_minimum_delay =
        shared_mutable_values.sdc.get_effective_minimum_delay_at(historical_block_number);
    shared_mutable_values.svc.get_block_horizon(historical_block_number, effective_minimum_delay)
}
