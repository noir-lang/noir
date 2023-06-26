
#[oracle(storageRead)]
fn storage_read_oracle(
    _storage_slot: Field,
) -> [Field; 1] {}

#[oracle(storageWrite)]
fn storage_write_oracle(
    _storage_slot: Field,
    _value: Field,
) -> [Field; 1] {}