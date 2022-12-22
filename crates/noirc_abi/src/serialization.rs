use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

use crate::AbiType;

// This module exposes a custom serializer and deserializer for `BTreeMap<String, AbiType>`
// (representing the fields of a struct) to serialize it as a `Vec<StructField>`.
//
// This is required as the struct is flattened into an array of field elements so the ordering of the struct's fields
// must be maintained. However, several serialization formats (notably JSON) do not provide strong guarantees about
// the ordering of elements in a map, this creates potential for improper ABI encoding of structs if the fields are
// deserialized into a different order. To prevent this, we store the fields in an array to create an unambiguous ordering.

#[derive(Serialize, Deserialize)]
struct StructField {
    name: String,
    #[serde(rename = "type")]
    typ: AbiType,
}

pub fn serialize_struct_fields<S>(
    fields: &BTreeMap<String, AbiType>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let fields_vector: Vec<StructField> = fields
        .iter()
        .map(|(name, typ)| StructField { name: name.to_owned(), typ: typ.to_owned() })
        .collect();
    fields_vector.serialize(s)
}

pub fn deserialize_struct_fields<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, AbiType>, D::Error>
where
    D: Deserializer<'de>,
{
    let fields_vector = Vec::<StructField>::deserialize(deserializer)?;
    let fields = fields_vector.into_iter().map(|StructField { name, typ }| (name, typ)).collect();
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{AbiParameter, AbiType, AbiVisibility, Sign};

    #[test]
    fn abi_parameter_serialization() {
        let serialized_field = "{
            \"name\": \"thing1\",
            \"visibility\": \"public\",
            \"type\": {
                \"kind\": \"field\"
            }
        }";

        let expected_field = AbiParameter {
            name: "thing1".to_string(),
            typ: AbiType::Field,
            visibility: AbiVisibility::Public,
        };
        let deserialized_field: AbiParameter = serde_json::from_str(serialized_field).unwrap();
        assert_eq!(deserialized_field, expected_field);

        let serialized_array = "{
            \"name\": \"thing2\",
            \"visibility\": \"private\",
            \"type\": {
                \"kind\": \"array\",
                \"length\": 2,
                \"type\": {
                    \"kind\": \"integer\",
                    \"width\": 3,
                    \"sign\": \"unsigned\"
                }
            }
        }";

        let expected_array = AbiParameter {
            name: "thing2".to_string(),
            typ: AbiType::Array {
                length: 2,
                typ: Box::new(AbiType::Integer { sign: Sign::Unsigned, width: 3 }),
            },
            visibility: AbiVisibility::Private,
        };
        let deserialized_array: AbiParameter = serde_json::from_str(serialized_array).unwrap();
        assert_eq!(deserialized_array, expected_array);

        let serialized_struct = "{   
            \"name\":\"thing3\",
            \"type\": {
                \"kind\":\"struct\",
                \"fields\": [
                    {
                        \"name\": \"field1\",
                        \"type\": {
                            \"kind\": \"integer\",
                            \"sign\": \"unsigned\",
                            \"width\": 3
                        }
                    },
                    {
                        \"name\":\"field2\",
                        \"type\": {
                            \"kind\":\"array\",
                            \"length\": 2,
                            \"type\": {
                                \"kind\":\"field\"
                            }
                        }
                    }
                ]
            },
            \"visibility\":\"private\"
        }";

        let expected_struct = AbiParameter {
            name: "thing3".to_string(),
            typ: AbiType::Struct {
                fields: BTreeMap::from([
                    ("field1".to_string(), AbiType::Integer { sign: Sign::Unsigned, width: 3 }),
                    (
                        "field2".to_string(),
                        AbiType::Array { length: 2, typ: Box::new(AbiType::Field) },
                    ),
                ]),
            },
            visibility: AbiVisibility::Private,
        };
        let deserialized_struct: AbiParameter = serde_json::from_str(serialized_struct).unwrap();
        assert_eq!(deserialized_struct, expected_struct);
    }
}
