use dep::protocol_types::traits::{Deserialize, Serialize};

#[oracle(storageRead)]
fn storage_read_oracle<N>(_storage_slot: Field, _number_of_elements: Field) -> [Field; N] {}

unconstrained fn storage_read_oracle_wrapper<N>(_storage_slot: Field) -> [Field; N] {
    storage_read_oracle(_storage_slot, N)
}

pub fn storage_read<N>(storage_slot: Field) -> [Field; N] {
    storage_read_oracle_wrapper(storage_slot)
}

#[oracle(storageWrite)]
fn storage_write_oracle<N>(_storage_slot: Field, _values: [Field; N]) -> [Field; N] {}

unconstrained pub fn storage_write<N>(storage_slot: Field, fields: [Field; N]) {
    let _hash = storage_write_oracle(storage_slot, fields);
}
