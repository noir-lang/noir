pub comptime fn size_in_fields(typ: Type) -> u32 {
    if typ.as_slice().is_some() {
        panic(f"Can't determine size in fields of Slice type")
    } else {
        let size = array_size_in_fields(typ);
        let size = size.or_else(|| struct_size_in_fields(typ));
        let size = size.or_else(|| tuple_size_in_fields(typ));
        size.unwrap_or(1)
    }
}

comptime fn array_size_in_fields(typ: Type) -> Option<u32> {
    typ.as_array().and_then(
        |typ: (Type, Type)| {
        let (element_size, typ) = typ;
        element_size.as_constant().map(|x: u32| {
            x * size_in_fields(typ)
        })
    }
    )
}

comptime fn struct_size_in_fields(typ: Type) -> Option<u32> {
    typ.as_struct().map(
        |typ: (StructDefinition, [Type])| {
        let struct_type = typ.0;
        let mut size = 0;
        for field in struct_type.fields() {
            size += size_in_fields(field.1);
        }
        size
    }
    )
}

comptime fn tuple_size_in_fields(typ: Type) -> Option<u32> {
    typ.as_tuple().map(
        |types: [Type]| {
        let mut size = 0;
        for typ in types {
            size += size_in_fields(typ);
        }
        size
    }
    )
}
