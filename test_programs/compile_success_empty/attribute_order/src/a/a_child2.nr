#[crate::assert_run_order_trait(2)]
pub trait Foo {}

#[crate::assert_run_order_function(3)]
pub fn foo() {}
