// The failing companion to `execution_success/validate_bounded`. `buffer` is decoded straight from
// the circuit inputs with `len = 5` while its capacity is only `3` — the invariant violation from
// https://github.com/noir-lang/noir-claude/issues/814. The `#[validate(Buffer::len_within_capacity)]`
// check the derived `validate` calls rejects it at runtime.
// This test is similar to `validate_bounded_vec_overflow`, but shows the derivation for a custom type.
use std::validate::validate;

#[derive(Validate)]
#[validate(Buffer::len_within_capacity)]
struct Buffer<let N: u32> {
    storage: [Field; N],
    len: u32,
}

impl<let N: u32> Buffer<N> {
    fn len_within_capacity(&self) {
        assert(self.len <= N, "Buffer length exceeds its capacity");
    }
}

fn main(buffer: Buffer<3>) {
    // Redundant: `buffer` is validated automatically on entry and fails there, before the body
    // runs, so an explicit call would never be reached.
    // buffer.validate();
    let _ = buffer;
}
