use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Mutex, OnceLock};

pub struct StaticHashCache<T: 'static + Eq + Hash>(OnceLock<Mutex<HashSet<&'static T>>>);

impl<T: 'static + Eq + Hash> StaticHashCache<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn intern(&self, value: T) -> &'static T {
        let cache = self.0.get_or_init(|| Mutex::new(HashSet::new()));
        let mut cache = cache.lock().unwrap();

        cache.get(&value).cloned().unwrap_or_else(|| {
            let interned = Box::leak(Box::new(value));
            cache.insert(interned);
            interned
        })
    }
}
