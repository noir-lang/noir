use crate::notes::value_note::Note;
use crate::types::point::Point;

#[oracle(storageRead)]
fn storage_read_oracle(
    _storage_slot: Field,
) -> Field {}

#[oracle(storageWrite)]
fn storage_write_oracle(
    _storage_slot: Field,
    _value: Field,
) -> Field {}

struct StorageField {
    storage_slot: Field,
}

impl StorageField {
    fn new(storage_slot: Field) -> StorageField {
        StorageField { storage_slot }
    }

    unconstrained fn read(self) -> Field {
        storage_read_oracle(self.storage_slot)
    }

    unconstrained fn write(self, value: Field) -> Field {
        storage_write_oracle(self.storage_slot, value)
    }
}
