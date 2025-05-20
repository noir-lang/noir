use super::traits::{Deserialize, Packable, Serialize};

/// Returns the typed expression of a trait method implementation.
///
/// This helper function is preferred over directly inlining with `$typ::target_method()` in a quote,
/// as direct inlining would result in missing import warnings in the generated code (specifically,
/// warnings that the trait implementation is not in scope).
///
/// # Note
/// A copy of this function exists in `aztec-nr/aztec/src/macros/utils.nr`. We maintain separate copies
/// because importing it there from here would cause the `target_trait` to be interpreted in the context
/// of this crate, making it impossible to compile code for traits from that crate (e.g. NoteType).
comptime fn get_trait_impl_method(
    typ: Type,
    target_trait: Quoted,
    target_method: Quoted,
) -> TypedExpr {
    let trait_constraint = target_trait.as_trait_constraint();
    typ
        .get_trait_impl(trait_constraint)
        .expect(f"Could not find impl for {target_trait} for type {typ}")
        .methods()
        .filter(|m| m.name() == target_method)[0]
        .as_typed_expr()
}

/// Generates code that deserializes a struct, primitive type, array or string from a field array.
///
/// # Parameters
/// - `name`: The name of the current field being processed, used to identify fields for replacement.
/// - `typ`: The type of the struct or field being deserialized (e.g., a custom struct, array, or primitive).
/// - `field_array_name`: The name of the field array containing serialized field data (e.g., `"values"`).
/// - `num_already_consumed`: The number of fields already processed in previous recursion calls.
/// - `should_unpack`: A boolean indicating whether the type should be unpacked (see description of `Packable`
/// and `Serialize` trait for more information about the difference between packing and serialization).
///
/// # Returns
/// A tuple containing:
/// - `Quoted`: A code that deserializes a given struct, primitive type, array, or string from the field array.
/// - `u32`: The total number of fields consumed during deserialization (used for recursion).
///
/// # Nested Struct Example
/// Given the following setup:
/// ```
/// struct UintNote {
///     value: u128,
///     owner: AztecAddress,
///     randomness: Field,
/// }
///
/// struct AztecAddress {
///     inner: Field,
/// }
/// ```
///
/// If `UintNote` is the input type, the function will generate the following deserialization code:
/// ```
/// UintNote {
///     value: fields[0] as u128,
///     owner: AztecAddress {
///         inner: fields[1],
///     },
///     randomness: fields[2],
/// }
/// ```
/// # Nested Struct Example with Unpacking
/// - given the same setup as above and given that u128, AztecAddress and Field implement the `Packable` trait
///   the result we get is:
/// ```
/// UintNote {
///     value: aztec::protocol_types::traits::Packable::unpack([fields[0]]),
///     owner: aztec::protocol_types::traits::Packable::unpack([fields[1]]),
///     randomness: aztec::protocol_types::traits::Packable::unpack([fields[2]]),
/// }
/// ```
///
/// # Panics
/// - If the deserialization logic encounters a type it does not support.
/// - If an incorrect number of fields are consumed when deserializing a string.
pub comptime fn generate_deserialize_from_fields(
    name: Quoted,
    typ: Type,
    field_array_name: Quoted,
    num_already_consumed: u32,
    should_unpack: bool,
) -> (Quoted, u32) {
    let mut result = quote {};
    // Counter for the number of fields consumed
    let mut consumed_counter: u32 = 0;

    // If the type implements `Packable`, its length will be assigned to the `maybe_packed_len_typ` variable.
    let maybe_packed_len_typ = std::meta::typ::fresh_type_variable();
    let packable_constraint = quote { Packable<$maybe_packed_len_typ> }.as_trait_constraint();

    if (should_unpack & typ.implements(packable_constraint)) {
        // Unpacking is enabled and the given type implements the `Packable` trait so we call the `unpack()`
        // method, add the resulting field array to `aux_vars` and each field to `fields`.
        let packed_len = maybe_packed_len_typ.as_constant().unwrap();

        // We copy the packed fields into a new array and pass that to the unpack function in a quote
        let mut packed_fields_quotes = &[];
        for i in 0..packed_len {
            let index_in_field_array = i + num_already_consumed;
            packed_fields_quotes =
                packed_fields_quotes.push_back(quote { $field_array_name[$index_in_field_array] });
        }
        let packed_fields = packed_fields_quotes.join(quote {,});

        // Now we call unpack on the type
        let unpack_method = get_trait_impl_method(typ, quote { Packable<_> }, quote { unpack });
        result = quote { $unpack_method([ $packed_fields ]) };

        consumed_counter = packed_len;
    } else if typ.is_field() | typ.as_integer().is_some() | typ.is_bool() {
        // The field is a primitive so we just reference it in the field array
        result = quote { $field_array_name[$num_already_consumed] as $typ };
        consumed_counter = 1;
    } else if typ.as_data_type().is_some() {
        // The field is a struct so we iterate over each struct field and recursively call
        // `generate_deserialize_from_fields`
        let (nested_def, generics) = typ.as_data_type().unwrap();
        let nested_name = nested_def.name();
        let mut deserialized_fields_list = &[];

        // Iterate over each field in the struct
        for field in nested_def.fields(generics) {
            let (field_name, field_type) = field;
            // Recursively call `generate_deserialize_from_fields` for each field in the struct
            let (deserialized_field, num_consumed_in_recursion) = generate_deserialize_from_fields(
                field_name,
                field_type,
                field_array_name,
                consumed_counter + num_already_consumed,
                should_unpack,
            );
            // We increment the consumed counter by the number of fields consumed in the recursion
            consumed_counter += num_consumed_in_recursion;
            // We add the deserialized field to the list of deserialized fields.
            // E.g. `value: u128 { lo: fields[0], hi: fields[1] }`
            deserialized_fields_list =
                deserialized_fields_list.push_back(quote { $field_name: $deserialized_field });
        }

        // We can construct the struct from the deserialized fields
        let deserialized_fields = deserialized_fields_list.join(quote {,});
        result = quote {
                $nested_name {
                    $deserialized_fields
                }
            };
    } else if typ.as_array().is_some() {
        // The field is an array so we iterate over each element and recursively call
        // `generate_deserialize_from_fields`
        let (element_type, array_len) = typ.as_array().unwrap();
        let array_len = array_len.as_constant().unwrap();
        let mut array_fields_list = &[];

        // Iterate over each element in the array
        for _ in 0..array_len {
            // Recursively call `generate_deserialize_from_fields` for each element in the array
            let (deserialized_field, num_consumed_in_recursion) = generate_deserialize_from_fields(
                name,
                element_type,
                field_array_name,
                consumed_counter + num_already_consumed,
                should_unpack,
            );
            // We increment the consumed counter by the number of fields consumed in the recursion
            consumed_counter += num_consumed_in_recursion;
            // We add the deserialized field to the list of deserialized fields.
            array_fields_list = array_fields_list.push_back(deserialized_field);
        }

        // We can construct the array from the deserialized fields
        let array_fields = array_fields_list.join(quote {,});
        result = quote { [ $array_fields ] };
    } else if typ.as_str().is_some() {
        // The field is a string and we expect each byte of the string to be represented as 1 field in the field
        // array. So we iterate over the string length and deserialize each character as u8 in the recursive call
        // to `generate_deserialize_from_fields`.
        let length_type = typ.as_str().unwrap();
        let str_len = length_type.as_constant().unwrap();
        let mut byte_list = &[];

        // Iterate over each character in the string
        for _ in 0..str_len {
            // Recursively call `generate_deserialize_from_fields` for each character in the string
            let (deserialized_field, num_consumed_in_recursion) = generate_deserialize_from_fields(
                name,
                quote {u8}.as_type(),
                field_array_name,
                consumed_counter + num_already_consumed,
                should_unpack,
            );

            // We should consume just one field in the recursion so we sanity check that
            assert_eq(
                num_consumed_in_recursion,
                1,
                "Incorrect number of fields consumed in string deserialization",
            );

            // We increment the consumed counter by 1 as we have consumed one field
            consumed_counter += 1;

            // We add the deserialized field to the list of deserialized fields.
            // E.g. `fields[6] as u8`
            byte_list = byte_list.push_back(deserialized_field);
        }

        // We construct the string from the deserialized fields
        let bytes = byte_list.join(quote {,});
        result = quote { [ $bytes ].as_str_unchecked() };
    } else {
        panic(
            f"Unsupported type for serialization of argument {name} and type {typ}",
        )
    }

    (result, consumed_counter)
}

/// Generates code that serializes a type into an array of fields. Also generates auxiliary variables if necessary
/// for serialization. If `should_pack` is true, we check if the type implements the `Packable` trait and pack it
/// if it does.
///
/// # Parameters
/// - `name`: The base identifier (e.g., `self`, `some_var`).
/// - `typ`: The type being serialized (e.g., a custom struct, array, or primitive type).
/// - `should_pack`: A boolean indicating whether the type should be packed.
///
/// # Returns
/// A tuple containing:
/// - A flattened array of `Quoted` field references representing the serialized fields.
/// - An array of `Quoted` auxiliary variables needed for serialization, such as byte arrays for strings.
///
/// # Examples
///
/// ## Struct
/// Given the following struct:
/// ```rust
/// struct MockStruct {
///     a: Field,
///     b: Field,
/// }
/// ```
///
/// Serializing the struct:
/// ```rust
/// generate_serialize_to_fields(quote { my_mock_struct }, MockStruct, false)
/// // Returns:
/// // ([`my_mock_struct.a`, `my_mock_struct.b`], [])
/// ```
///
/// ## Nested Struct
/// For a more complex struct:
/// ```rust
/// struct NestedStruct {
///     m1: MockStruct,
///     m2: MockStruct,
/// }
/// ```
///
/// Serialization output:
/// ```rust
/// generate_serialize_to_fields(quote { self }, NestedStruct, false)
/// // Returns:
/// // ([`self.m1.a`, `self.m1.b`, `self.m2.a`, `self.m2.b`], [])
/// ```
///
/// ## Array
/// For an array type:
/// ```rust
/// generate_serialize_to_fields(quote { my_array }, [Field; 3], false)
/// // Returns:
/// // ([`my_array[0]`, `my_array[1]`, `my_array[2]`], [])
/// ```
///
/// ## String
/// For a string field, where each character is serialized as a `Field`:
/// ```rust
/// generate_serialize_to_fields(quote { my_string }, StringType, false)
/// // Returns:
/// // ([`my_string_as_bytes[0] as Field`, `my_string_as_bytes[1] as Field`, ...],
/// // [`let my_string_as_bytes = my_string.as_bytes()`])
/// ```
///
/// ## Nested Struct with packing enabled
/// - u128 has a `Packable` implementation hence it will be packed.
///
/// For a more complex struct:
/// ```rust
/// struct MyStruct {
///     value: u128,
///     value2: Field,
/// }
/// ```
///
/// # Panics
/// - If the type is unsupported for serialization.
/// - If the provided `typ` contains invalid constants or incompatible structures.
pub comptime fn generate_serialize_to_fields(
    name: Quoted,
    typ: Type,
    should_pack: bool,
) -> ([Quoted], [Quoted]) {
    let mut fields = &[];
    let mut aux_vars = &[];

    // If the type implements `Packable`, its length will be assigned to the `maybe_packed_len_typ` variable.
    let maybe_packed_len_typ = std::meta::typ::fresh_type_variable();
    let packable_constraint =
        quote { crate::traits::Packable<$maybe_packed_len_typ> }.as_trait_constraint();

    if (should_pack & typ.implements(packable_constraint)) {
        // Packing is enabled and the given type implements the `Packable` trait so we call the `pack()`
        // method, add the resulting field array to `aux_vars` and each field to `fields`.
        let packed_len = maybe_packed_len_typ.as_constant().unwrap();

        // We collapse the name to a one that gets tokenized as a single token (e.g. "self.value" -> "self_value").
        let name_at_one_token = collapse_to_one_token(name);
        let packed_struct_name = f"{name_at_one_token}_aux_var".quoted_contents();

        // We add the individual fields to the fields array
        let pack_method = get_trait_impl_method(
            typ,
            quote { crate::traits::Packable<$packed_len> },
            quote { pack },
        );
        let packed_struct = quote { let $packed_struct_name = $pack_method($name) };
        for i in 0..packed_len {
            fields = fields.push_back(quote { $packed_struct_name[$i] });
        }

        // We add the new auxiliary variable to the aux_vars array
        aux_vars = aux_vars.push_back(packed_struct);
    } else if typ.is_field() {
        // For field we just add the value to fields
        fields = fields.push_back(name);
    } else if typ.as_integer().is_some() | typ.is_bool() {
        // For integer and bool we just cast to Field and add the value to fields
        fields = fields.push_back(quote { $name as Field });
    } else if typ.as_data_type().is_some() {
        // For struct we pref
        let nested_struct = typ.as_data_type().unwrap();
        let params = nested_struct.0.fields(nested_struct.1);
        let struct_flattened = params.map(|(param_name, param_type): (Quoted, Type)| {
            let maybe_prefixed_name = if name == quote {} {
                // Triggered when the param name is of a value available in the current scope (e.g. a function
                // argument) --> then we don't prefix the name with anything.
                param_name
            } else {
                // Triggered when we want to prefix the param name with the `name` from function input. This
                // can typically be `self` when implementing a method on a struct.
                quote { $name.$param_name }
            };
            generate_serialize_to_fields(quote {$maybe_prefixed_name}, param_type, should_pack)
        });
        let struct_flattened_fields = struct_flattened.fold(
            &[],
            |acc: [Quoted], (fields, _): (_, [Quoted])| acc.append(fields),
        );
        let struct_flattened_aux_vars = struct_flattened.fold(
            &[],
            |acc: [Quoted], (_, aux_vars): ([Quoted], _)| acc.append(aux_vars),
        );
        fields = fields.append(struct_flattened_fields);
        aux_vars = aux_vars.append(struct_flattened_aux_vars);
    } else if typ.as_array().is_some() {
        // For array we recursively call `generate_serialize_to_fields(...)` for each element
        let (element_type, array_len) = typ.as_array().unwrap();
        let array_len = array_len.as_constant().unwrap();
        for i in 0..array_len {
            let (element_fields, element_aux_vars) =
                generate_serialize_to_fields(quote { $name[$i] }, element_type, should_pack);
            fields = fields.append(element_fields);
            aux_vars = aux_vars.append(element_aux_vars);
        }
    } else if typ.as_str().is_some() {
        // For string we convert the value to bytes, we store the `as_bytes` in an auxiliary variables and
        // then we add each byte to fields as a Field
        let length_type = typ.as_str().unwrap();
        let str_len = length_type.as_constant().unwrap();
        let as_member = name.as_expr().unwrap().as_member_access();
        let var_name = if as_member.is_some() {
            as_member.unwrap().1
        } else {
            name
        };
        let as_bytes_name = f"{var_name}_as_bytes".quoted_contents();
        let as_bytes = quote { let $as_bytes_name = $name.as_bytes() };
        for i in 0..str_len {
            fields = fields.push_back(quote { $as_bytes_name[$i] as Field });
        }
        aux_vars = aux_vars.push_back(as_bytes);
    } else {
        panic(
            f"Unsupported type for serialization of argument {name} and type {typ}",
        )
    }

    (fields, aux_vars)
}

/// From a quote that gets tokenized to a multiple tokens we collapse it to a single token by replacing all `.` with `_`.
/// E.g. "self.values[0]" -> "self_values_0_"
comptime fn collapse_to_one_token(q: Quoted) -> Quoted {
    let tokens = q.tokens();

    let mut single_token = quote {};
    for token in tokens {
        let new_token = if ((token == quote {.}) | (token == quote {[}) | (token == quote {]})) {
            quote {_}
        } else {
            token
        };
        single_token = f"{single_token}{new_token}".quoted_contents();
    }
    single_token
}

pub(crate) comptime fn derive_serialize(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    let (fields, aux_vars) = generate_serialize_to_fields(quote { self }, typ, false);
    let aux_vars_for_serialization = if aux_vars.len() > 0 {
        let joint = aux_vars.join(quote {;});
        quote { $joint; }
    } else {
        quote {}
    };

    let field_serializations = fields.join(quote {,});
    let serialized_len = fields.len();
    quote {
        impl Serialize<$serialized_len> for $typ {
            fn serialize(self) -> [Field; $serialized_len] {
                $aux_vars_for_serialization
                [ $field_serializations ]
            }
        }
    }
}

pub(crate) comptime fn derive_deserialize(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    let (fields, _) = generate_serialize_to_fields(quote { self }, typ, false);
    let serialized_len = fields.len();
    let (deserialized, _) =
        generate_deserialize_from_fields(quote { self }, typ, quote { serialized }, 0, false);
    quote {
        impl Deserialize<$serialized_len> for $typ {
            fn deserialize(serialized: [Field; $serialized_len]) -> Self {
                $deserialized
            }
        }
    }
}

/// Generates `Packable` implementation for a given struct and returns the packed length.
///
/// Note: We are having this function separate from `derive_packable` because we use this in the note macros to get
/// the packed length of a note as well as the `Packable` implementation. We need the length to be able to register
/// the note in the global `NOTES` map. There the length is used to generate partial note helper functions.
pub comptime fn derive_packable_and_get_packed_len(s: TypeDefinition) -> (Quoted, u32) {
    let packing_enabled = true;

    let typ = s.as_type();
    let (fields, aux_vars) = generate_serialize_to_fields(quote { self }, typ, packing_enabled);
    let aux_vars_for_packing = if aux_vars.len() > 0 {
        let joint = aux_vars.join(quote {;});
        quote { $joint; }
    } else {
        quote {}
    };

    let (unpacked, _) =
        generate_deserialize_from_fields(quote { self }, typ, quote { packed }, 0, packing_enabled);

    let field_packings = fields.join(quote {,});
    let packed_len = fields.len();
    let packable_trait: TraitConstraint = quote { Packable<$packed_len> }.as_trait_constraint();
    (
        quote {
        impl $packable_trait for $typ {
            fn pack(self) -> [Field; $packed_len] {
                $aux_vars_for_packing
                [ $field_packings ]
            }

            fn unpack(packed: [Field; $packed_len]) -> Self {
                $unpacked
            }
        }
    },
        packed_len,
    )
}

pub(crate) comptime fn derive_packable(s: TypeDefinition) -> Quoted {
    let (packable_impl, _) = derive_packable_and_get_packed_len(s);
    packable_impl
}

#[derive(Packable, Serialize, Deserialize, Eq)]
pub struct Smol {
    a: Field,
    b: Field,
}

#[derive(Serialize, Deserialize, Eq)]
pub struct HasArray {
    a: [Field; 2],
    b: bool,
}

#[derive(Serialize, Deserialize, Eq)]
pub struct Fancier {
    a: Smol,
    b: [Field; 2],
    c: [u8; 3],
    d: str<16>,
}

fn main() {
    assert(false);
}

#[test]
fn smol_test() {
    let smol = Smol { a: 1, b: 2 };
    let serialized = smol.serialize();
    assert(serialized == [1, 2], serialized);
    let deserialized = Smol::deserialize(serialized);
    assert(deserialized == smol);

    // None of the struct members implements the `Packable` trait so the packed and serialized data should be the same
    let packed = smol.pack();
    assert_eq(packed, serialized, "Packed does not match serialized");
}

#[test]
fn has_array_test() {
    let has_array = HasArray { a: [1, 2], b: true };
    let serialized = has_array.serialize();
    assert(serialized == [1, 2, 1], serialized);
    let deserialized = HasArray::deserialize(serialized);
    assert(deserialized == has_array);
}

#[test]
fn fancier_test() {
    let fancier =
        Fancier { a: Smol { a: 1, b: 2 }, b: [0, 1], c: [1, 2, 3], d: "metaprogramming!" };
    let serialized = fancier.serialize();
    assert(
        serialized
            == [
                1, 2, 0, 1, 1, 2, 3, 0x6d, 0x65, 0x74, 0x61, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61,
                0x6d, 0x6d, 0x69, 0x6e, 0x67, 0x21,
            ],
        serialized,
    );
    let deserialized = Fancier::deserialize(serialized);
    assert(deserialized == fancier);
}
