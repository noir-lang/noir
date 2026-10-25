use macros::expect_registered;

// `wrapper` and this package must agree on which `LibThing` they mean.
pub fn take(_thing: notelib::LibThing) {}

pub fn use_wrapper() {
    take(wrapper::make());
}

#[expect_registered(1)]
pub struct CheckFour {}
