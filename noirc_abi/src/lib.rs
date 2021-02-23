// This is the ABI used to bridge the different TOML formats for the initial
// witness, the partial witness generator and the interpreter.
//
// This ABI has nothing to do with ACVM or ACIR. Although they implicitly have a relationship

// XXX: Unfortunately this is still linked to Aztec's protocol
// the ordering of public inputs, is due to the barretenberg standard format API
// where we need public inputs to be added initially

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
    Private,
    Public,
    Array { length: u128, typ: Box<AbiType> },
    Integer { length: u128, typ: Box<AbiType> },
}

impl AbiType {
    pub fn num_elements(&self) -> usize {
        match self {
            AbiType::Private | AbiType::Public | AbiType::Integer { .. } => 1,
            AbiType::Array { length, typ: _ } => *length as usize,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Abi {
    pub parameters: Vec<(String, AbiType)>,
}

impl Abi {
    // In barretenberg, we need to add public inputs first
    // currently there does not seem to be a way to add a witness and then a public input
    // So we have this special function to sort for barretenberg.
    // It will need to be abstracted away or hidden behind the aztec_backend
    pub fn sort_by_public_input(mut self) -> Self {
        let comparator = |a: &(String, AbiType), b: &(String, AbiType)| {
            let typ_a = &a.1;
            let typ_b = &b.1;

            if typ_a == &AbiType::Public && typ_b == &AbiType::Public {
                std::cmp::Ordering::Equal
            } else if typ_a == &AbiType::Public {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        };

        self.parameters.sort_by(comparator);
        self
    }

    pub fn parameter_names(&self) -> Vec<&String> {
        self.parameters.iter().map(|x| &x.0).collect()
    }

    pub fn len(&self) -> usize {
        self.parameters
            .iter()
            .map(|(_, param_type)| param_type.num_elements())
            .sum()
    }
}
