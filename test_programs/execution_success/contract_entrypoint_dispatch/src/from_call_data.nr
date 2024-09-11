/// A trait for types that can be decoded from a calldata field array
#[derive_via(derive_from_call_data)]
trait FromCallData<let N: u32> {
    /// Decodes Self by reading it from calldata at the given index, also returning the next index to read from
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field);
}

comptime fn derive_from_call_data(s: StructDefinition) -> Quoted {
    let typ = s.as_type();
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
            impl <let N: u32> FromCallData<N> for $typ {
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
