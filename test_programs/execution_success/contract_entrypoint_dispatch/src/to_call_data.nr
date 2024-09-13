/// A trait for types that can be serialized to a field array
#[derive_via(derive_to_call_data)]
trait ToCallData<let N: u32> {
    /// Serializes self into calldata at the given index, returning the next index to write to
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field;
}

comptime fn derive_to_call_data(s: StructDefinition) -> Quoted {
    let typ = s.as_type();

    let impl_generics = s.generics().map(|g| quote { $g });
    let impl_generics = impl_generics.push_front(quote { let N: u32 });
    let impl_generics = impl_generics.join(quote {,});

    let where_clause = s.generics().map(|name| quote { $name: ToCallData<N> }).join(quote {,});

    let names = s.fields().map(|f: (Quoted, Type)| f.0);
    if names.len() == 0 {
        quote {
            impl <let N: u32> ToCallData<N> for $typ {
                fn to_calldata(_self: Self, _calldata: &mut [Field; N], index: Field) -> Field {
                    index
                }
            }
        }
    } else {
        let statements = names.map(
            |name: Quoted| {
            quote { 
                let index = ToCallData::to_calldata(self.$name, calldata, index);
            }
        }
        );
        let statements = statements.join(quote { });
        quote {
            impl <$impl_generics> ToCallData<N> for $typ where $where_clause {
                fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
                    $statements
                    index
                }
            }
        }
    }
}

impl <let N: u32> ToCallData<N> for Field {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for bool {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for u1 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for u8 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for u16 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for u32 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for u64 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for i8 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for i16 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for i32 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32> ToCallData<N> for i64 {
    fn to_calldata(self, calldata: &mut [Field; N], index: Field) -> Field {
        calldata[index] = self as Field;
        index + 1
    }
}

impl <let N: u32, T, let M: u32> ToCallData<N> for [T; M] where T: ToCallData<N> {
    fn to_calldata(self, calldata: &mut [Field; N], mut index: Field) -> Field {
        for loop_index in 0..M {
            index = ToCallData::to_calldata(self[loop_index], calldata, index);
        }
        index
    }
}

mod tests {
    use super::ToCallData;

    #[derive(ToCallData)]
    struct Point<T> {
        x: T,
        y: T,
    }

    #[test]
    fn test_field_to_call_data() {
        let mut calldata = [0];
        let value: Field = 1;
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [1]);
        assert_eq(index, 1);
    }

    #[test]
    fn test_bool_to_call_data() {
        let mut calldata = [0];
        let value = true;
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [1]);
        assert_eq(index, 1);

        let mut calldata = [1];
        let value = false;
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [0]);
        assert_eq(index, 1);
    }

    #[test]
    fn test_u8_to_call_data() {
        let mut calldata = [0];
        let value: u8 = 1;
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [1]);
        assert_eq(index, 1);
    }

    #[test]
    fn test_struct_to_call_data() {
        let mut calldata = [0, 0];
        let value = Point { x: 1, y: 2 };
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [1, 2]);
        assert_eq(index, 2);
    }

    #[test]
    fn test_array_to_call_data() {
        let mut calldata = [0, 0, 0];
        let value = [1, 2, 3];
        let index = ToCallData::to_calldata(value, &mut calldata, 0);
        assert_eq(calldata, [1, 2, 3]);
        assert_eq(index, 3);
    }
}
