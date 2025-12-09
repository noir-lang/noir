pub comptime fn derive_serialize(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    let nested_struct = typ.as_data_type().unwrap();
    let params = nested_struct.0.fields(nested_struct.1);

    // The following will give us [<type_of_struct_member_1 as Serialize>::N, <type_of_struct_member_2 as Serialize>::N, ...]
    let components_of_definition_of_n = params.map(|(_, param_type, _): (Quoted, Type, Quoted)| {
        quote {
            <$param_type as Serialize>::N
        }
    });

    // The following will give us <type_of_struct_member_1 as Serialize>::N + <type_of_struct_member_2 as Serialize>::N + ...
    let right_hand_side_of_definition_of_n = components_of_definition_of_n.join(quote {+});

    let array_of_quotes_serializing_each_struct_member = params
        .map(|(param_name, param_type, _): (Quoted, Type, Quoted)| {
            quote {
                let serialized_member = self.$param_name.serialize();
                let serialized_member_len = <$param_type as Serialize>::N;
                for i in 0..serialized_member_len {
                    result[i + offset] = serialized_member[i];
                }
                offset += serialized_member_len;
            }
        });

    let serialization_of_struct_members =
        array_of_quotes_serializing_each_struct_member.join(quote {});

    quote {
        impl Serialize for $typ {
            let N: u32 = $right_hand_side_of_definition_of_n;

            #[inline_always]
            fn serialize(self) -> [Field; Self::N] {
                let mut result = [0; _];
                let mut offset = 0;

                $serialization_of_struct_members

                result
            }
        }
    }
}

pub(crate) comptime fn derive_deserialize(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    let nested_struct = typ.as_data_type().unwrap();
    let params = nested_struct.0.fields(nested_struct.1);
    // The following will give us [<type_of_struct_member_1 as Deserialize>::N, <type_of_struct_member_2 as Deserialize>::N, ...]
    let components_of_definition_of_n = params.map(|(_, param_type, _): (Quoted, Type, Quoted)| {
        quote {
            <$param_type as Deserialize>::N
        }
    });

    // The following will give us <type_of_struct_member_1 as Deserialize>::N + <type_of_struct_member_2 as Deserialize>::N + ...
    let serialized_len = components_of_definition_of_n.join(quote {+});

    let array_of_quotes_deserializing_each_struct_member = params
        .map(|(param_name, param_type, _): (Quoted, Type, Quoted)| {
            quote {
                let mut member_fields = [0; <$param_type as Deserialize>::N];
                for i in 0..<$param_type as Deserialize>::N {
                    member_fields[i] = serialized[i + offset];
                }
                let $param_name = <$param_type as Deserialize>::deserialize(member_fields);
                offset += <$param_type as Deserialize>::N;
            }
        });

    let deserialization_of_struct_members =
        array_of_quotes_deserializing_each_struct_member.join(quote {});

    // We join the struct member names with a comma to be used in the `Self { ... }` syntax
    // This will give us e.g. `a, b, c` for a struct with three fields named `a`, `b`, and `c`.
    let struct_members = params
        .map(|(param_name, _, _): (Quoted, Type, Quoted)| quote { $param_name })
        .join(quote {,});

    quote {
        impl Deserialize for $typ {
            let N: u32 = $serialized_len;

            #[inline_always]
            fn deserialize(serialized: [Field; Self::N]) -> Self {
                let mut offset = 0;
                $deserialization_of_struct_members
                Self { $struct_members }
            }
        }
    }
}
