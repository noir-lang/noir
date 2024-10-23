// This is run fifth, after each child module finishes even though
// the function comes before the module declarations in the source.
#[crate::assert_run_order_function(4)]
fn foo() {}

mod a_child1;
mod a_child2;

#[crate::assert_run_order_struct(5)]
struct Bar {}

#[crate::assert_run_order_trait(6)]
trait Foo {}
