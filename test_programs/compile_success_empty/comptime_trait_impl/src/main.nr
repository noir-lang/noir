use std::meta::type_of;

trait SomeTrait<X, Y> {

}
struct SomeStruct {

}

impl SomeTrait<i32, i64> for SomeStruct {

}

fn main() {
    comptime
    {
        let some_struct = quote { SomeStruct }.as_type();
        let some_trait = quote { SomeTrait<i32, i64> }.as_trait_constraint();
        let trait_impl = some_struct.get_trait_impl(some_trait).unwrap();

        // Check TraitImpl::trait_generic_args
        let trait_generic_args = trait_impl.trait_generic_args();
        assert_eq(trait_generic_args.len(), 2);
        assert_eq(trait_generic_args[0], quote { i32 }.as_type());
        assert_eq(trait_generic_args[1], quote { i64 }.as_type());
    }
}
