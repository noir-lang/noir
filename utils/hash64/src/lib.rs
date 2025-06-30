/// Utility to return the 64-bit hash of a value using rustc-hash.
pub fn hash64<T: std::hash::Hash + ?Sized>(v: &T) -> u64 {
    use std::hash::Hasher;
    let mut state = rustc_hash::FxHasher::default();
    v.hash(&mut state);
    state.finish()
}
