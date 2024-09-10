fn main() {
    comptime
    {
        // Check UnresolvedType::is_bool
        let typ = quote { x as bool }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.is_bool());

        // Check UnresolvedType::is_field
        let typ = quote { x as Field }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.is_field());
    }
}
