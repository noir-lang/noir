//! Encoding of the [`FuzzerData`] test cases a fuzz target keeps in its corpus.
//!
//! A target's `fuzz_target!` and `fuzz_mutator!` callbacks both receive a raw
//! `&[u8]`: the mutator writes the bytes that the target reads back on the next
//! iteration, but nothing in the type system relates the two. Both callbacks of
//! a target therefore go through one [`CorpusCodec`] value, which names the wire
//! format in a single place and fixes the decoded type by signature rather than
//! by inference from a fallback value. A target that reads a different type than
//! its mutator writes still fuzzes, but every case decodes to
//! `FuzzerData::default()`, which describes no instructions and so compiles and
//! compares nothing.
#![allow(dead_code)] // Each fuzz target uses one codec; the other is unused in that binary.

use super::fuzzer::FuzzerData;

/// Serialization format of a target's corpus entries.
///
/// The choice is per target and must not change without discarding that
/// target's corpus, whose files are all in the old format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CorpusCodec {
    /// `serde_json`. Human-readable, so a corpus entry or crash can be read and
    /// hand-edited directly.
    Json,
    /// `rmp_serde` (MessagePack). Compact, which keeps entries under
    /// libFuzzer's `-max_len` for larger programs.
    MessagePack,
}

impl CorpusCodec {
    /// Serializes a test case for storage in the corpus.
    pub(crate) fn encode(self, data: &FuzzerData) -> Vec<u8> {
        match self {
            Self::Json => serde_json::to_vec(data).expect("FuzzerData serializes as JSON"),
            Self::MessagePack => {
                rmp_serde::encode::to_vec(data).expect("FuzzerData serializes as MessagePack")
            }
        }
    }

    /// Deserializes a corpus entry, describing the failure if the bytes are not
    /// a test case in this format.
    pub(crate) fn decode(self, bytes: &[u8]) -> Result<FuzzerData, String> {
        match self {
            Self::Json => serde_json::from_slice(bytes).map_err(|error| error.to_string()),
            Self::MessagePack => {
                rmp_serde::decode::from_slice(bytes).map_err(|error| error.to_string())
            }
        }
    }

    /// Deserializes a corpus entry, falling back to [`FuzzerData::default`] so a
    /// mutator always has something to mutate.
    ///
    /// Only for the mutator: the default describes no instructions, so a target
    /// that runs it compiles and compares nothing.
    pub(crate) fn decode_or_default(self, bytes: &[u8]) -> FuzzerData {
        match self.decode(bytes) {
            Ok(data) => data,
            Err(error) => {
                log::debug!("Mutating the default test case; {} bytes: {error}", bytes.len());
                FuzzerData::default()
            }
        }
    }
}
