use crate::context::{Context};
use dep::protocol_types::traits::{Deserialize, Serialize};

trait Storage<T> where T: Serialize<N> + Deserialize<N> {
    fn get_storage_slot(self) -> Field {
        self.storage_slot
    }
}
