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
    x: usize,
    accumulated: usize,
    disabled: usize,
}

impl Freq {
    pub(crate) fn new(u: &mut Unstructured, freqs: &Freqs) -> arbitrary::Result<Self> {
        let x = u.choose_index(freqs.total())?;
        Ok(Self { freqs: freqs.clone(), x, accumulated: 0, disabled: 0 })
    }

    /// Check if a key is enabled, based on the already checked cumulative values.
    ///
    /// To work correctly, `enabled` calls should come after `enabled_when`s.
    pub(crate) fn enabled(&mut self, key: &str) -> bool {
        self.accumulated += self.freqs[key];
        self.passed()
    }

    /// Like `enabled`, but if `cond` is `false` it redistributes the probability
    /// to the remaining keys, pretending to works as if the current one never existed.
    ///
    /// To work correctly, `enabled_when` calls should precede `enabled`s.
    ///
    /// It's not guaranteed to prevent any distortions; for example it can happen that
    /// the first key we check is enabled by `cond`, but we skip it because `x` is higher
    /// than its weight, only to then realize that all following keys are disabled.
    /// In this case the weight of the first check was different than if we considered
    /// all conditions up front, summed up the remaining weights, and then chose `x`
    /// accordingly. Doing so could potentially slow things down.
    ///
    /// As a workaround, we should
    /// * check keys which are more likely to be disabled up front
    /// * have a default case at the end which is always enabled
    pub(crate) fn enabled_when(&mut self, key: &str, cond: bool) -> bool {
        if cond {
            self.enabled(key)
        } else {
            self.disabled += self.freqs[key];
            false
        }
    }

    /// Check if the accumulated weights have passed the random variable `x`,
    /// adjusted with the total disabled weight.
    fn passed(&self) -> bool {
        if self.freqs.total == 0 {
            return false;
        }
        let adj_total = self.freqs.total - self.disabled;
        let adj_x = self.x * adj_total / self.freqs.total;
        self.accumulated > adj_x
    }
}

#[cfg(test)]
mod tests {
    use super::{Freq, Freqs};

    /// Create a test distribution with a given `x in 0..100`.
    fn make_test_freq(x: usize) -> Freq {
        assert!(x < 100, "scale goes to 100");
        let freqs = Freqs::new(&[("foo", 40), ("bar", 30), ("baz", 20), ("qux", 10)]);
        Freq { freqs, accumulated: 0, disabled: 0, x }
    }

    /// Test that it enables baz, because the cumulative freq including baz is 90.
    #[test]
    fn test_freq_enabled() {
        let mut freq = make_test_freq(80);
        assert!(!freq.enabled("foo"));
        assert!(!freq.enabled("bar"));
        assert!(freq.enabled("baz"));
    }

    /// Test that changing the order results in a different label being enabled,
    /// because the cross the threshold at a different call.
    #[test]
    fn test_freq_enabled_order() {
        let mut freq = make_test_freq(80);
        assert!(!freq.enabled("baz"));
        assert!(!freq.enabled("foo"));
        assert!(freq.enabled("bar"));
    }

    /// Test that some labels being disabled redistribute the probability
    /// on the remaining ones. In this example we choose x=75 out of 100,
    /// which is the 75th percentile, but disable the first 70 weights,
    /// which means if we want `x` to be the 75th percentile of the
    /// remaining 30 we adjust it down to 75*30/100 = 22, which enables
    /// qux az instead of baz, which would have originally fallen between [70,90).
    #[test]
    fn test_freq_redistribution() {
        let mut freq = make_test_freq(75);
        assert!(!freq.enabled_when("foo", false));
        assert!(!freq.enabled_when("bar", false));
        assert!(!freq.enabled_when("baz", false));
        assert!(freq.enabled("qux"));
    }
}
