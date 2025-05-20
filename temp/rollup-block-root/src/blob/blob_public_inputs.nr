// TODO(MW): delete

// NB: This only exists because a nested array of [[Field; 2]; N] did not build with ci2.5, but was fine otherwise
// For blobs, we use the compressed 48 byte BLS12 commitment to compute the challenge. We never need to operate on it,
// so it's encoded as 2 fields. The first is the first 31 bytes, and the second is the next 17 bytes.
pub(crate) struct BlobCommitment {
    pub(crate) inner: [Field; 2],
}
