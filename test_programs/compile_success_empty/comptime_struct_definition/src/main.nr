#[my_comptime_fn]
pub struct MyType<A, B, C> {
    field1: [A; 10],
    field2: (B, C),
}

#[mutate_struct_fields]
pub struct I32AndField {
    z: i8,
}

comptime fn my_comptime_fn(typ: TypeDefinition) {
    let _ = typ.as_type();
    assert_eq(typ.generics().len(), 3);
    assert_eq(typ.fields_as_written().len(), 2);
    assert_eq(typ.name(), quote { MyType });
}

comptime fn mutate_struct_fields(s: TypeDefinition) {
    let fields = [
        (quote {x}, quote {i32}.as_type(), quote { pub }),
        (quote {y}, quote {Field}.as_type(), quote { pub(crate) }),
    ].as_vector();
    s.set_fields(fields);

    let fields = s.fields([]);
    assert_eq(fields[0].2, quote { pub });
    assert_eq(fields[1].2, quote { pub(crate) });

    let fields = s.fields_as_written();
    assert_eq(fields[0].2, quote { pub });
    assert_eq(fields[1].2, quote { pub(crate) });
}

mod foo {
    #[attr]
    pub struct Foo {}

    comptime fn attr(s: TypeDefinition) {
        assert_eq(s.module().name(), quote { foo });
    }

    #[add_generic]
    pub struct Bar {}

    // docs:start:add-generic-example
    comptime fn add_generic(s: TypeDefinition) {
        assert_eq(s.generics().len(), 0);
        let new_generic = s.add_generic("T");

        let generics = s.generics();
        assert_eq(generics.len(), 1);
        let (typ, numeric) = generics[0];
        assert_eq(typ, new_generic);
        assert(numeric.is_none());
    }
    // docs:end:add-generic-example
}

fn main() {
    comptime {
        let typ = quote { MyType<i8, i16, i32> }.as_type();
        let (struct_def, generics) = typ.as_data_type().unwrap();

        let fields = struct_def.fields(generics);
        assert_eq(fields.len(), 2);

        let (field1_name, field1_type, visibility1) = fields[0];
        let (field2_name, field2_type, visibility2) = fields[1];

        assert_eq(field1_name, quote { field1 });
        assert_eq(field2_name, quote { field2 });

        // Ensure .fields(generics) actually performs substitutions on generics
        assert_eq(field1_type, quote { [i8; 10] }.as_type());
        assert_eq(field2_type, quote { (i16, i32) }.as_type());

        assert_eq(visibility1, quote {});
        assert_eq(visibility2, quote {});

        // Test TypeDefinition::as_type_with_generics
        let i32_type = quote [i32].as_type();

        let applied = struct_def.as_type_with_generics([i32_type, i32_type, i32_type].as_vector());
        let invalid = struct_def.as_type_with_generics([i32_type, i32_type].as_vector()); // invalid generic count

        assert_eq(applied.unwrap(), quote [MyType<i32, i32, i32>].as_type());
        assert(invalid.is_none());
    }
}
