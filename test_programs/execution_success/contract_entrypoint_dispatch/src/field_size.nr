pub comptime fn size_in_fields(typ: Type) -> u32 {
    if typ.is_bool() | typ.is_field() {
        1
    } else {
        let typ_as_struct = typ.as_struct();
        if typ_as_struct.is_some() {
            let (struct_type, _) = typ_as_struct.unwrap();
            let mut size = 0;
            for field in struct_type.fields() {
                size += size_in_fields(field.1);
            }
            size
        } else {
            // TODO: do the reamining types
            0
        }
    }
}
