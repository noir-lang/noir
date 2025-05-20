use crate::{
    constants::{
        AVM_VERIFICATION_KEY_LENGTH_IN_FIELDS, CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS,
        HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS, ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS,
    },
    traits::{Deserialize, Empty, Serialize},
};

pub struct VerificationKey<let N: u32> {
    pub key: [Field; N],
    pub hash: Field,
}

impl<let N: u32> VerificationKey<N> {
    pub fn check_hash(self) {
        assert_eq(self.hash, crate::hash::verification_key_hash(self.key), "Invalid VK hash");
    }
}

impl<let N: u32> Serialize<N + 1> for VerificationKey<N> {
    fn serialize(self) -> [Field; N + 1] {
        let mut fields = [0; N + 1];
        for i in 0..N {
            fields[i] = self.key[i];
        }
        fields[N] = self.hash;
        fields
    }
}

impl<let N: u32> Deserialize<N + 1> for VerificationKey<N> {
    fn deserialize(fields: [Field; N + 1]) -> Self {
        let mut key = VerificationKey::empty();
        for i in 0..N {
            key.key[i] = fields[i];
        }
        key.hash = fields[N];
        key
    }
}

impl<let N: u32> Empty for VerificationKey<N> {
    fn empty() -> Self {
        VerificationKey { hash: 0, key: [0; N] }
    }
}

impl<let N: u32> Eq for VerificationKey<N> {
    fn eq(self, other: Self) -> bool {
        (self.hash == other.hash) & (self.key == other.key)
    }
}

pub type RollupHonkVerificationKey = VerificationKey<ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>;
pub type HonkVerificationKey = VerificationKey<HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>;
pub type ClientIVCVerificationKey = VerificationKey<CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS>;
pub type AvmVerificationKey = VerificationKey<AVM_VERIFICATION_KEY_LENGTH_IN_FIELDS>;

#[test]
fn serialization_of_empty() {
    let key: VerificationKey<20> = VerificationKey::empty();
    let serialized = key.serialize();
    let deserialized = VerificationKey::deserialize(serialized);
    assert(key.eq(deserialized));
}
