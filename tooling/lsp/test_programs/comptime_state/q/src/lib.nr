use macros::expect_registered;

// Nothing in this package's dependencies registers, so nothing must be visible here,
// even though `x` is also used by `p3` next to `notelib`.
#[expect_registered(0)]
pub struct CheckQ {}
