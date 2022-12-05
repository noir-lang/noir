use std::convert::TryInto;

use serde::{Deserialize, Serialize, Serializer};

use crate::{Abi, AbiFEType, AbiType, Sign};

// Serializing the `Abi` struct directly doesn't result in the most convenient serialized format for outside use.
// To fix this we convert `Abi` into a `Vec<AbiParameter>` before serialization as this avoids having to write a custom
// serializer and deserializer.

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AbiParameter {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<AbiFEType>,
    #[serde(rename = "type")]
    param_type: Type,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Type {
    Field,
    Array {
        length: u64,
        #[serde(rename = "type")]
        typ: Box<Type>,
    },
    Integer {
        sign: Sign,
        width: u32,
    },
    Struct {
        fields: Vec<AbiParameter>,
    },
}

impl From<AbiType> for Type {
    fn from(param_type: AbiType) -> Self {
        match param_type {
            AbiType::Field(_) => Self::Field,
            AbiType::Array { visibility: _, length, typ } => {
                Self::Array { length: length.try_into().unwrap(), typ: Box::new(Self::from(*typ)) }
            }
            AbiType::Integer { visibility: _, sign, width } => Self::Integer { sign, width },
            AbiType::Struct { visibility: _, fields } => Self::Struct {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| AbiParameter {
                        name,
                        visibility: None,
                        param_type: Type::from(value),
                    })
                    .collect(),
            },
        }
    }
}

impl AbiType {
    fn from_type(param_type: Type, visibility: AbiFEType) -> Self {
        match param_type {
            Type::Field => AbiType::Field(visibility),
            Type::Integer { sign, width } => AbiType::Integer { visibility, sign, width },
            Type::Array { length, typ } => AbiType::Array {
                visibility,
                length: length.into(),
                typ: Box::new(Self::from_type(*typ, visibility)),
            },
            Type::Struct { fields } => AbiType::Struct {
                visibility,
                fields: fields
                    .into_iter()
                    .map(|field| (field.name, AbiType::from_type(field.param_type, visibility)))
                    .collect(),
            },
        }
    }
}

impl Serialize for Abi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let parameter_vec: Vec<AbiParameter> = self
            .parameters
            .clone()
            .into_iter()
            .map(|(name, param_type)| {
                let visibility = Some(param_type.visibility());
                AbiParameter { name, visibility, param_type: Type::from(param_type) }
            })
            .collect();

        parameter_vec.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Abi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let parameters = Vec::<AbiParameter>::deserialize(deserializer)?;

        let parameters = parameters
            .into_iter()
            .map(|AbiParameter { name, visibility, param_type }| {
                (name, AbiType::from_type(param_type, visibility.unwrap()))
            })
            .collect();

        Ok(Abi { parameters })
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{Abi, AbiFEType, AbiType, Sign};

    #[test]
    fn abi_deserializes_correctly() {
        let serialized_abi = "
        [
            {
                \"name\": \"thing1\",
                \"visibility\": \"public\",
                \"type\": {
                    \"kind\": \"field\"
                }
            },
            {
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
            },
            {
                \"name\": \"thing3\",
                \"visibility\": \"private\",
                \"type\": {
                    \"kind\": \"struct\",
                    \"fields\": [
                        {
                            \"name\": \"field1\",
                            \"type\": {
                                \"kind\": \"integer\",
                                \"width\": 3,
                                \"sign\": \"unsigned\"
                            }
                        },
                        {
                            \"name\": \"field2\",
                            \"type\": {
                                \"kind\": \"array\",
                                \"length\": 2,
                                \"type\": {
                                    \"kind\": \"field\"
                                }
                            }
                        }
                    ]
                }
            }
        ]";

        let deserialized_abi: Abi = serde_json::from_str(serialized_abi).unwrap();

        let mut params = deserialized_abi.parameters.iter();
        assert_eq!(params.next(), Some(&("thing1".to_string(), AbiType::Field(AbiFEType::Public))));
        assert_eq!(
            params.next(),
            Some(&(
                "thing2".to_string(),
                AbiType::Array {
                    visibility: AbiFEType::Private,
                    length: 2,
                    typ: Box::new(AbiType::Integer {
                        visibility: AbiFEType::Private,
                        sign: Sign::Unsigned,
                        width: 3
                    })
                }
            ))
        );
        assert_eq!(
            params.next(),
            Some(&(
                "thing3".to_string(),
                AbiType::Struct {
                    visibility: AbiFEType::Private,
                    fields: BTreeMap::from([
                        (
                            "field1".to_string(),
                            AbiType::Integer {
                                visibility: AbiFEType::Private,
                                sign: Sign::Unsigned,
                                width: 3
                            }
                        ),
                        (
                            "field2".to_string(),
                            AbiType::Array {
                                visibility: AbiFEType::Private,
                                length: 2,
                                typ: Box::new(AbiType::Field(AbiFEType::Private))
                            }
                        )
                    ])
                }
            ))
        );
    }
}
