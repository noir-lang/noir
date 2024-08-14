use crate::keys::public_keys::{PublicKeys, PUBLIC_KEYS_LENGTH};
use dep::protocol_types::traits::{Serialize, Deserialize};

// This struct represents how public keys are stored in the key registry. We store not just the keys themselves but also
// their hash, so that when reading in private we can perform a historical read for the hash and then show that it
// corresponds to a preimage obtained from an unconstrained hint. We do store the keys keys regardless as they might be
// needed during public execution, and since we need to broadcast and produce hints in some standardized way.
// While it might seem odd to create a struct for what is effectively some data and a pure function called on it, state
// variables rely on serializable structs in order to persist data to storage, so we must use this abstraction.
struct StoredKeys {
    public_keys: PublicKeys,
    hash: Field,
}

impl StoredKeys {
    // Instances of StoredKeys are expected to only be created by calling this function so that we guarantee that the
    // hash field does indeed correspond to the hash of the keys. Ideally we'd forbid direct access to the struct, but 
    // Noir doesn't yet support private members.
    fn new(public_keys: PublicKeys) -> Self {
        Self { public_keys, hash: public_keys.hash().inner }
    }
}

// Our serialization is the concatenation of the public keys serialization plush the hash, so we need one extra field.
global STORED_KEYS_LENGTH: u32 =  PUBLIC_KEYS_LENGTH + 1;

impl Serialize<STORED_KEYS_LENGTH> for StoredKeys {
    fn serialize(self) -> [Field; STORED_KEYS_LENGTH] {
        // The code below is equivalent to:
        // [ ...self.public_keys.serialize(), self.hash ]

        let mut array = [0; STORED_KEYS_LENGTH];

        let serialized_keys = self.public_keys.serialize();
        for i in 0..serialized_keys.len() {
            array[i] = serialized_keys[i];
        }

        array[PUBLIC_KEYS_LENGTH] = self.hash;

        array
    }
}

impl Deserialize<STORED_KEYS_LENGTH> for StoredKeys {
    fn deserialize(array: [Field; STORED_KEYS_LENGTH]) -> Self {
        // The code below is equivalent to:
        // Self { public_keys: PublicKeys::deserialize(array[0 : PUBLIC_KEYS_LENGTH]), hash: array[PUBLIC_KEYS_LENGTH] }

        let mut serialized_keys = [0; PUBLIC_KEYS_LENGTH];
        for i in 0..serialized_keys.len() {
            serialized_keys[i] = array[i];
        }

        let hash = array[PUBLIC_KEYS_LENGTH];

        Self { public_keys: PublicKeys::deserialize(serialized_keys), hash }
    }
}
