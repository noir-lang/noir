use crate::hash::{Hasher, pedersen_hash};
use crate::default::Default;

struct PedersenHasher{
    _state: [Field]
}

impl Hasher for PedersenHasher {
    fn finish(self) -> Field {
       pedersen_hash(self._state)
    }

    fn write(self, input: [Field]) -> Self{
        let _state = self._state.append(input);
        PedersenHasher{_state}
    }
}

impl Default for PedersenHasher{
    fn default() -> Self{
        PedersenHasher{
            _state: []
        }
    }
}
