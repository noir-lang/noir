fn main() {
    comptime {
        // Check UnresolvedType::is_bool
        let typ = quote { x as bool }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.is_bool());

        // Check UnresolvedType::is_field
        let typ = quote { x as Field }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.is_field());

        // Check UnresolvedType::is_unit
        let typ = quote { x as () }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.is_unit());

        // Check UnresolvedType::as_mutable_reference
        let typ = quote { x as &mut Field }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.as_mutable_reference().unwrap().is_field());

        // Check UnresolvedType::as_slice
        let typ = quote { x as [Field] }.as_expr().unwrap().as_cast().unwrap().1;
        assert(typ.as_slice().unwrap().is_field());
    }
}
