#[crate::assert_run_order_trait(2)]
trait Foo {}

#[crate::assert_run_order_function(3)]
fn foo() {}
