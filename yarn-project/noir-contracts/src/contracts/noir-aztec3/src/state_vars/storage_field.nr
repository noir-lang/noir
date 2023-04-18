use crate::notes::value_note::Note;
use crate::types::point::Point;

#[builtin(storageRead)]
fn storage_read(
    _storage_slot: Field,
) -> Field {}

#[builtin(storageWrite)]
fn storage_write(
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

    fn read(self) -> Field {
        storage_read(self.storage_slot)
    }

    fn write(self, value: Field) -> Field {
        storage_write(self.storage_slot, value)
    }
}
