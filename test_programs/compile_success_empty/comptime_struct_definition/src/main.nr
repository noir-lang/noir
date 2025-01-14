#[my_comptime_fn]
pub struct MyType<A, B, C> {
    field1: [A; 10],
    field2: (B, C),
}

#[mutate_struct_fields]
pub struct I32AndField {
    z: i8,
}

comptime fn my_comptime_fn(typ: StructDefinition) {
    let _ = typ.as_type();
    assert_eq(typ.generics().len(), 3);
    assert_eq(typ.fields_as_written().len(), 2);
    assert_eq(typ.name(), quote { MyType });
}

comptime fn mutate_struct_fields(s: StructDefinition) {
    let fields = &[(quote {x}, quote {i32}.as_type()), (quote {y}, quote {Field}.as_type())];
    s.set_fields(fields);
}

mod foo {
    #[attr]
    pub struct Foo {}

    comptime fn attr(s: StructDefinition) {
        assert_eq(s.module().name(), quote { foo });
    }

    #[add_generic]
    pub struct Bar {}

    // docs:start:add-generic-example
    comptime fn add_generic(s: StructDefinition) {
        assert_eq(s.generics().len(), 0);
        let new_generic = s.add_generic("T");

        let generics = s.generics();
        assert_eq(generics.len(), 1);
        assert_eq(generics[0], new_generic);
    }
    // docs:end:add-generic-example
}

fn main() {
    comptime {
        let typ = quote { MyType<i8, i16, i32> }.as_type();
        let (struct_def, generics) = typ.as_struct().unwrap();

        let fields = struct_def.fields(generics);
        assert_eq(fields.len(), 2);

        let (field1_name, field1_type) = fields[0];
        let (field2_name, field2_type) = fields[1];
        
        assert_eq(field1_name, quote { field1 });
        assert_eq(field2_name, quote { field2 });

        // Ensure .fields(generics) actually performs substitutions on generics
        assert_eq(field1_type, quote { [i8; 10] }.as_type());
        assert_eq(field2_type, quote { (i16, i32) }.as_type());
    }
}
