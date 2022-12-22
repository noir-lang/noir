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
