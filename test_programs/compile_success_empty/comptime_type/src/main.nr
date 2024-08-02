use std::meta::type_of;

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
    }
}
