use crate::hash::{Hasher, pedersen_hash_slice};
use crate::default::Default;

struct PedersenHasher{
    _state: [Field]
}

impl Hasher for PedersenHasher {
    fn finish(self) -> Field {
       pedersen_hash_slice(self._state)
    }

    fn write(&mut self, input: [Field]){
        self._state = self._state.append(input);
    }
}

impl Default for PedersenHasher{
    fn default() -> Self{
        PedersenHasher{
            _state: &[]
        }
    }
}
