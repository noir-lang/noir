use acir_field::AcirField;
use serde::{Deserialize, Serialize};

/// Single input or output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[serde(untagged)]
pub enum ForeignCallParam<F> {
    Single(F),
    Array(Vec<F>),
}

impl<F> From<F> for ForeignCallParam<F> {
    fn from(value: F) -> Self {
        ForeignCallParam::Single(value)
    }
}

impl<F> From<Vec<F>> for ForeignCallParam<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallParam::Array(values)
    }
}

impl<F: AcirField> ForeignCallParam<F> {
    /// Convert the fields in the parameter into a vector, used to flatten data.
    pub fn fields(&self) -> Vec<F> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.to_vec(),
        }
    }

    pub fn unwrap_field(&self) -> F {
        match self {
            ForeignCallParam::Single(value) => *value,
            ForeignCallParam::Array(_) => panic!("Expected single value, found array"),
        }
    }
}

/// Represents the full output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct ForeignCallResult<F> {
    /// Resolved output values of the foreign call.
    pub values: Vec<ForeignCallParam<F>>,
}

impl<F> From<F> for ForeignCallResult<F> {
    fn from(value: F) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

impl<F> From<Vec<F>> for ForeignCallResult<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

impl<F> From<Vec<ForeignCallParam<F>>> for ForeignCallResult<F> {
    fn from(values: Vec<ForeignCallParam<F>>) -> Self {
        ForeignCallResult { values }
    }
}

#[cfg(test)]
mod test {
    use acir_field::FieldElement;
    use proptest::prelude::*;

    use super::{ForeignCallParam, ForeignCallResult};

    // Define a wrapper around field so we can implement `Arbitrary`.
    // NB there are other methods like `arbitrary_field_elements` around the codebase,
    // but for `proptest_derive::Arbitrary` we need `F: AcirField + Arbitrary`.
    acir_field::field_wrapper!(TestField, FieldElement);

    impl Arbitrary for TestField {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any::<u128>().prop_map(|v| Self(FieldElement::from(v))).boxed()
        }
    }

    proptest! {
        #[test]
        fn foreign_call_arg_serialization_roundtrip(param: ForeignCallParam<TestField>) {
            let serialized = serde_json::to_string(&param).unwrap();
            let deserialized: ForeignCallParam<TestField> = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(param, deserialized);
        }

        #[test]
        fn foreign_call_return_serialization_roundtrip(param: ForeignCallResult<TestField>) {
            let serialized = serde_json::to_string(&param).unwrap();
            let deserialized: ForeignCallResult<TestField> = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(param, deserialized);
        }
    }
}
