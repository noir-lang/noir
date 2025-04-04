use std::ops::Index;

use arbitrary::Unstructured;

/// Frequency distribution of generators.
#[derive(Debug, Clone)]
pub struct Freqs {
    items: im::HashMap<&'static str, usize>,
    total: usize,
}

impl Freqs {
    pub fn new(items: &[(&'static str, usize)]) -> Self {
        let total = items.iter().map(|i| i.1).sum();
        Self { items: items.iter().cloned().collect(), total }
    }
    pub fn total(&self) -> usize {
        self.total
    }
}

impl Index<&str> for Freqs {
    type Output = usize;

    fn index(&self, index: &str) -> &Self::Output {
        self.items.get(index).unwrap_or_else(|| panic!("unknown freq: {index}"))
    }
}

/// Help with cumulative frequency distributions.
pub(crate) struct Freq {
    freqs: Freqs,
    acc: usize,
    x: usize,
}

impl Freq {
    pub fn new(u: &mut Unstructured, freqs: &Freqs) -> arbitrary::Result<Self> {
        Ok(Self { freqs: freqs.clone(), acc: 0, x: u.choose_index(freqs.total())? })
    }

    /// Check if a key is enabled, based on the already checked cumulative values.
    pub fn enabled(&mut self, key: &str) -> bool {
        self.acc += self.freqs[key];
        self.x < self.acc
    }

    /// Like `enabled`, but if `cond` is `false` it does not increase the cumulative value,
    /// so as not to distort the next call, ie. if we have have 5% then another 5%,
    /// if the first one is disabled, the second doesn't become 10%.
    pub fn enabled_if(&mut self, key: &str, cond: bool) -> bool {
        cond && self.enabled(key)
    }
}
