// This is run fifth, after each child module finishes even though
// the function comes before the module declarations in the source.
#[crate::assert_run_order_function(4)]
pub fn foo() {}

mod a_child1;
mod a_child2;

#[crate::assert_run_order_struct(5)]
pub struct Bar {}

#[crate::assert_run_order_trait(6)]
pub trait Foo {}
