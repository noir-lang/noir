/// A trait for types that can be deserialized from a calldata field array
#[derive_via(derive_from_call_data)]
trait FromCallData<let N: u32> {
    /// Deserializes Self by reading it from calldata at the given index, also returning the next index to read from
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field);
}

comptime fn derive_from_call_data(s: StructDefinition) -> Quoted {
    let typ = s.as_type();

    let impl_generics = s.generics().map(|g| quote { $g });
    let impl_generics = impl_generics.push_front(quote { let N: u32 });
    let impl_generics = impl_generics.join(quote {,});

    let where_clause = s.generics().map(|name| quote { $name: FromCallData<N> }).join(quote {,});

    let names = s.fields().map(|f: (Quoted, Type)| f.0);
    if names.len() == 0 {
        quote {
            impl <let N: u32> FromCallData<N> for $typ {
                fn from_calldata(_calldata: [Field; N], index: Field) -> (Self, Field) {
                    (Self {}, index)
                }
            }
        }
    } else {
        let statements = names.map(
            |name: Quoted| {
            quote { 
                let ($name, index) = FromCallData::from_calldata(calldata, index);
            }
        }
        );
        let self_names = names.join(quote { , });
        let statements = statements.join(quote { });
        quote {
            impl <$impl_generics> FromCallData<N> for $typ where $where_clause {
                fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
                    $statements
                    (Self { $self_names }, index)
                }
            }
        }
    }
}

impl <let N: u32> FromCallData<N> for Field {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index];
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for bool {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as bool;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for u1 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as u1;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for u8 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as u8;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for u16 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as u16;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for u32 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as u32;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for u64 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as u64;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for i8 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as i8;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for i16 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as i16;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for i32 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as i32;
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for i64 {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index] as i64;
        (value, index + 1)
    }
}

impl <let N: u32, T, let M: u32> FromCallData<N> for [T; M] where T: FromCallData<N> {
    fn from_calldata(calldata: [Field; N], mut index: Field) -> (Self, Field) {
        let mut values = [std::mem::zeroed(); M];
        for loop_index in 0..M {
            let (value, new_index) = FromCallData::from_calldata(calldata, index);
            values[loop_index] = value;
            index = new_index;
        }
        (values, index)
    }
}

mod tests {
    use super::FromCallData;

    #[derive(FromCallData)]
    struct Point<T> {
        x: T,
        y: T,
    }

    #[test]
    fn test_field_from_call_data() {
        let calldata = [1];
        let (value, index): (Field, Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value, 1);
        assert_eq(index, 1);
    }

    #[test]
    fn test_bool_from_call_data() {
        let calldata = [1];
        let (value, index): (bool, Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value, true);
        assert_eq(index, 1);

        let calldata = [0];
        let (value, index): (bool, Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value, false);
        assert_eq(index, 1);
    }

    #[test]
    fn test_u8_from_call_data() {
        let calldata = [1];
        let (value, index): (u8, Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value, 1);
        assert_eq(index, 1);
    }

    #[test]
    fn test_struct_from_call_data() {
        let calldata = [1, 2];
        let (value, index): (Point<Field>, Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value.x, 1);
        assert_eq(value.y, 2);
        assert_eq(index, 2);
    }

    #[test]
    fn test_array_from_call_data() {
        let calldata = [1, 2, 3];
        let (value, index): ([Field; 3], Field) = FromCallData::from_calldata(calldata, 0);
        assert_eq(value, [1, 2, 3]);
        assert_eq(index, 3);
    }
}
