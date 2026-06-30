pub fn safe_call(x: Field) -> Field {
    // Library invariant: x in {0, 1}. The range check lives only in the public API.
    assert((x == 0) | (x == 1));
    internal::raw_compute(x)
}

mod internal {
    // Private submodule. Trusts the invariant; emits no range check of its own.
    pub(crate) fn raw_compute(x: Field) -> Field {
        x * 1000
    }
}
