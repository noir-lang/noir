use crate::notes::value_note::Note;
use crate::types::point::Point;

// oracle
use crate::oracle::storage::storage_read_oracle;
use crate::oracle::storage::storage_write_oracle;

struct StorageField {
    storage_slot: Field,
}

impl StorageField {
    fn new(storage_slot: Field) -> StorageField {
        StorageField { storage_slot }
    }

    unconstrained fn read(self) -> Field {
        storage_read_oracle(self.storage_slot)[0]
    }

    unconstrained fn write(self, value: Field) -> Field {
        storage_write_oracle(self.storage_slot, value)[0]
    }
}
