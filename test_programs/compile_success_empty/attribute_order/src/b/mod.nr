// Declare these in reverse order to ensure the declaration
// order here is respected
mod b_child2;

#[crate::assert_run_order_module(10)]
mod b_child1;

#[crate::assert_run_order_function(11)]
pub fn foo() {}

#![crate::assert_run_order_module(12)]

#[crate::assert_run_order_function(13)]
pub fn bar() {}

#![crate::assert_run_order_module(14)]
