#[crate::assert_run_order_function(0)]
pub fn foo() {}

#[crate::assert_run_order_struct(1)]
pub struct Foo {}
