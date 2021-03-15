use serde::{ser::SerializeMap, Serialize, Serializer};

// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

pub mod input_parser;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Types that are allowed in the (main function in binary)
///
/// we use this separation so that we can have types like Strings
/// without needing to introduce this in the Noir types
///
/// NOTE: If Strings are introduced as a native type, the translation will
/// be straightforward. Whether exotic types like String will be natively supported
/// depends on the types of programs that users want to do. I don't envision string manipulation
/// in programs, however it is possible to support, with many complications like encoding character set
/// support.
pub enum AbiType {
    Field(AbiFEType),
    Array {
        visibility: AbiFEType,
        length: u128,
        typ: Box<AbiType>,
    },
    Integer {
        visibility: AbiFEType,
        sign: Sign,
        width: u32,
    },
}
/// This is the same as the FieldElementType in AST, without constants.
/// We don't want the ABI to depend on Noir, so types are not shared between the two
/// Note: At the moment, it is not even possible since the ABI is in another crate and Noir depends on it
/// This can be easily fixed by making the ABI a module.
///
/// In the future, maybe it will be decided that the AST will hold esoteric types and the HIR will transform them
/// This method is a bit cleaner as we would not need to dig into the resolver, to lower from a esoteric AST type to a HIR type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbiFEType {
    Public,
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    Private,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Unsigned,
    Signed,
}

impl AbiType {
    pub fn num_elements(&self) -> usize {
        match self {
            AbiType::Field(_) | AbiType::Integer { .. } => 1,
            AbiType::Array {
                visibility: _,
                length,
                typ: _,
            } => *length as usize,
        }
    }

    pub fn is_public(&self) -> bool {
        match self {
            AbiType::Field(fe_type) => fe_type == &AbiFEType::Public,
            AbiType::Array {
                visibility,
                length: _,
                typ: _,
            } => visibility == &AbiFEType::Public,
            AbiType::Integer {
                visibility,
                sign: _,
                width: _,
            } => visibility == &AbiFEType::Public,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Abi {
    pub parameters: Vec<(String, AbiType)>,
}

impl Abi {
    pub fn parameter_names(&self) -> Vec<&String> {
        self.parameters.iter().map(|x| &x.0).collect()
    }

    pub fn num_parameters(&self) -> usize {
        self.parameters.len()
    }
    /// Abi with only the public parameters
    pub fn public_abi(self) -> Abi {
        let parameters: Vec<_> = self
            .parameters
            .into_iter()
            .filter(|(_, param_type)| param_type.is_public())
            .collect();
        Abi { parameters }
    }
}

impl Serialize for Abi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<u8> = Vec::new();
        let mut map = serializer.serialize_map(Some(self.parameters.len()))?;
        for (param_name, param_type) in &self.parameters {
            match param_type {
                AbiType::Field(_) => map.serialize_entry(&param_name, "")?,
                AbiType::Array { .. } => map.serialize_entry(&param_name, &vec)?,
                AbiType::Integer { .. } => map.serialize_entry(&param_name, "")?,
            };
        }
        map.end()
    }
}
