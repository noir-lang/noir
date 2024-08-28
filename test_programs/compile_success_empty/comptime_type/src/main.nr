use std::meta::type_of;

struct Foo<T> {
    x: T
}

trait SomeTrait<T> {

}
struct StructImplementsSomeTrait {

}

impl SomeTrait<i32> for StructImplementsSomeTrait {

}

struct StructDoesNotImplementSomeTrait {

}

fn main() {
    comptime
    {
        // Check type_of works correctly (relies on Eq for Type)
        let a_field = 0;
        let another_field = 1;
        let an_i32: i32 = 0;
        let field_type_1 = type_of(a_field);
        let field_type_2 = type_of(another_field);
        let i32_type = type_of(an_i32);
        assert(field_type_1 == field_type_2);
        assert(field_type_1 != i32_type);

        // Check Type::is_field
        assert(field_type_1.is_field());
        assert(!i32_type.is_field());

        // Check Type::as_integer
        assert(field_type_1.as_integer().is_none());

        let (signed, bits) = i32_type.as_integer().unwrap();
        assert(signed);
        assert_eq(bits, 32);

        let a_u8: u8 = 0;
        let u8_type = type_of(a_u8);
        let (signed, bits) = u8_type.as_integer().unwrap();
        assert(!signed);
        assert_eq(bits, 8);

        // Check Type::as_tuple
        assert(u8_type.as_tuple().is_none());

        let tuple = (an_i32, a_u8);
        let tuple_type = type_of(tuple);
        let tuple_types = tuple_type.as_tuple().unwrap();
        assert_eq(tuple_types.len(), 2);
        assert_eq(tuple_types[0], i32_type);
        assert_eq(tuple_types[1], u8_type);

        // Check Type::as_slice
        assert(u8_type.as_slice().is_none());

        let slice = &[1];
        let slice_type = type_of(slice);
        let slice_type_element_type = slice_type.as_slice().unwrap();
        assert_eq(slice_type_element_type, field_type_1);

        // Check Type::as_array
        assert(u8_type.as_array().is_none());

        let array = [1, 2, 3];
        let array_type = type_of(array);
        let (array_type_element_type , array_length) = array_type.as_array().unwrap();
        assert_eq(array_type_element_type, field_type_1);

        // Check Type::as_constant
        assert(u8_type.as_constant().is_none());
        assert_eq(array_length.as_constant().unwrap(), 3);

        // Check Type::is_bool
        assert(!u8_type.is_bool());

        let yes = true;
        let bool_type = type_of(yes);
        assert(bool_type.is_bool());

        // Check Type::as_struct
        assert(u8_type.as_struct().is_none());

        let foo = Foo { x: 0 };
        let foo_type = type_of(foo);
        let (struct_definition, generics) = foo_type.as_struct().unwrap();
        let fields = struct_definition.fields();
        assert_eq(fields.len(), 1);

        assert_eq(generics.len(), 1);
        assert_eq(generics[0], field_type_1);

        // Check Type::implements
        let some_trait_i32 = quote { SomeTrait<i32> }.as_trait_constraint();
        let struct_implements_some_trait = quote { StructImplementsSomeTrait }.as_type();
        let struct_does_not_implement_some_trait = quote { StructDoesNotImplementSomeTrait }.as_type();
        assert(struct_implements_some_trait.implements(some_trait_i32));
        assert(!struct_does_not_implement_some_trait.implements(some_trait_i32));

        let some_trait_field = quote { SomeTrait<Field> }.as_trait_constraint();
        assert(!struct_implements_some_trait.implements(some_trait_field));
        assert(!struct_does_not_implement_some_trait.implements(some_trait_field));

        let _trait_impl = struct_implements_some_trait.get_trait_impl(some_trait_i32).unwrap();
    }
}

// docs:start:implements_example
fn function_with_where<T>(_x: T) where T: SomeTrait<i32> {
    comptime
    {
        let t = quote { T }.as_type();
        let some_trait_i32 = quote { SomeTrait<i32> }.as_trait_constraint();
        assert(t.implements(some_trait_i32));

        assert(t.get_trait_impl(some_trait_i32).is_none());
    }
}
// docs:end:implements_example
