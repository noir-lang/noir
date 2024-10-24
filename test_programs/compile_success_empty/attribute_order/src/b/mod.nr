// Declare these in reverse order to ensure the declaration
// order here is respected
mod b_child2;

#[crate::assert_run_order_module(12)]
mod b_child1;

#[crate::assert_run_order_function(13)]
pub fn foo() {}

#![crate::assert_run_order_module(14)]

// Test inline submodules as well
#[crate::assert_run_order_module(15)]
mod b_child3 {
    #![crate::assert_run_order_module(10)]

    #[crate::assert_run_order_function(11)]
    pub fn foo() {}
}

#[crate::assert_run_order_function(16)]
pub fn bar() {}

#![crate::assert_run_order_module(17)]
