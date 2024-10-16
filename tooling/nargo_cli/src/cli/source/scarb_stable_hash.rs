use std::hash::{Hash, Hasher};

use data_encoding::BASE32_DNSSEC;
use xxhash_rust::xxh3::Xxh3;

/// Implementation of a hasher that produces the same values across Scarb releases.
///
/// The hasher should be fast and have a low chance of collisions (but is not sufficient for
/// cryptographic purposes).
#[derive(Default)]
pub struct StableHasher(Xxh3);

impl StableHasher {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn finish_as_short_hash(&self) -> String {
        let hash = self.finish();
        BASE32_DNSSEC.encode(&hash.to_le_bytes())
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
}

pub fn short_hash(hashable: impl Hash) -> String {
    let mut hasher = StableHasher::new();
    hashable.hash(&mut hasher);
    hasher.finish_as_short_hash()
}

#[cfg(test)]
mod tests {
    use super::short_hash;

    #[test]
    fn short_hash_is_stable() {
        assert_eq!(short_hash("abcd"), "e1p6jp2ak1nmk");
        assert_eq!(short_hash(123), "8fupdqgl2ulsq");
    }
}
