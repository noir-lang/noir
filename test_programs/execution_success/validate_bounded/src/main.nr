// The `#[validate(Type::method)]` opt-in expressing a bounded-container invariant. `Buffer<N>` has
// the same shape as `BoundedVec` (a fixed-capacity backing array plus a length), and its invariant
// — `len <= N` — is exactly the one a value decoded from an untrusted source could violate (see
// https://github.com/noir-lang/noir-claude/issues/814). `#[derive(Validate)]` recurses into the
// backing array; `#[validate(Buffer::len_within_capacity)]` adds the cross-field length check the
// recursion cannot see.
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
    buffer.validate();
}
