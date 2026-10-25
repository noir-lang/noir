use macros::expect_registered;

// notelib registers once; x registers nothing.
#[expect_registered(1)]
pub struct CheckThree {}
