use acir_field::AcirField;
use serde::{Deserialize, Serialize};

/// Single input or output of a [foreign call][crate::Opcode::ForeignCall].
///
/// This enum can represent either a single field element or an array of field elements.
///
/// The `#[serde(untagged)]` attribute uses the natural JSON representation:
/// `Single(5)` serializes as `5`, and `Array([1,2,3])` serializes as `[1,2,3]`.
/// This allows external systems to pass values directly without needing to know about
/// Rust enum variants, relying on JSON's native distinction between single values and arrays.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ForeignCallParam<F> {
    /// A single field element value.
    Single(F),
    /// Multiple field element values (array or vector passed by value).
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
    /// Convert the parameter into a uniform vector representation.
    ///
    /// This is useful for flattening data when processing foreign call results,
    /// allowing both `Single` and `Array` variants to be handled uniformly as `Vec<F>`.
    pub fn fields(&self) -> Vec<F> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.to_vec(),
        }
    }

    /// Unwrap the field in a `Single` input. Panics if it's an `Array`.
    pub fn unwrap_field(&self) -> F {
        match self {
            ForeignCallParam::Single(value) => *value,
            ForeignCallParam::Array(_) => panic!("Expected single value, found array"),
        }
    }
}

/// Represents the full output of a [foreign call][crate::Opcode::ForeignCall].
///
/// A foreign call can return multiple outputs, where each output can be either
/// a single field element or an array. This struct wraps a vector of [ForeignCallParam]
/// to represent all outputs from a single foreign call.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct ForeignCallResult<F> {
    /// Resolved output values of the foreign call.
    ///
    /// Each element represents one output, which can be either a single value or an array.
    pub values: Vec<ForeignCallParam<F>>,
}

/// Convenience conversion for a foreign call returning a single field element value.
///
/// Creates a `ForeignCallResult` with one output containing the single value.
impl<F> From<F> for ForeignCallResult<F> {
    fn from(value: F) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

/// Conversion for a foreign call returning a single array output.
/// Creates a `ForeignCallResult` with one output, where that output is an array.
/// This represents one array output, not multiple single-value outputs:
///
/// ```ignore
/// let result: ForeignCallResult<F> = vec![1, 2, 3].into();
/// // result.values.len() == 1
/// // result.values[0] == ForeignCallParam::Array([1, 2, 3])
/// ```
///
/// For multiple single-value outputs, use `From<Vec<ForeignCallParam<F>>>` instead:
/// ```ignore
/// let result: ForeignCallResult<F> = vec![
///     ForeignCallParam::Single(1),
///     ForeignCallParam::Single(2),
///     ForeignCallParam::Single(3)
/// ].into();
/// // result.values.len() == 3
/// ```
impl<F> From<Vec<F>> for ForeignCallResult<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

/// Conversion for a foreign call returning multiple outputs.
///
/// Each element in the vector represents a separate output, which can be
/// either a single value or an array.
impl<F> From<Vec<ForeignCallParam<F>>> for ForeignCallResult<F> {
    fn from(values: Vec<ForeignCallParam<F>>) -> Self {
        ForeignCallResult { values }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir_field::FieldElement;

    #[test]
    fn test_foreign_call_param_from_single() {
        let value = FieldElement::from(42u128);
        let param = ForeignCallParam::from(value);

        assert_eq!(param, ForeignCallParam::Single(value));
        assert_eq!(param.fields(), vec![value]);
        assert_eq!(param.unwrap_field(), value);
    }

    #[test]
    fn test_foreign_call_param_from_array() {
        let values =
            vec![FieldElement::from(1u128), FieldElement::from(2u128), FieldElement::from(3u128)];
        let param = ForeignCallParam::from(values.clone());

        assert_eq!(param, ForeignCallParam::Array(values.clone()));
        assert_eq!(param.fields(), values);
    }

    #[test]
    fn test_foreign_call_param_array_roundtrip() {
        // Test that Array variant round trips through From<Vec<F>> and fields()
        let original = vec![
            FieldElement::from(10u128),
            FieldElement::from(20u128),
            FieldElement::from(30u128),
        ];

        // Need explicit type annotation to disambiguate from From<F> impl
        let param: ForeignCallParam<FieldElement> = original.clone().into();
        let roundtrip = param.fields();

        assert_eq!(roundtrip, original);
    }

    #[test]
    fn test_foreign_call_param_single_to_array() {
        // Note: Single doesn't roundtrip perfectly because fields() returns Vec
        // This test documents the expected behavior
        let value = FieldElement::from(42u128);
        let param = ForeignCallParam::Single(value);

        // fields() converts Single to a Vec with one element
        assert_eq!(param.fields(), vec![value]);
    }

    #[test]
    #[should_panic(expected = "Expected single value, found array")]
    fn test_foreign_call_param_unwrap_field_panics_on_array() {
        let param =
            ForeignCallParam::Array(vec![FieldElement::from(1u128), FieldElement::from(2u128)]);

        // This should panic
        param.unwrap_field();
    }

    #[test]
    fn test_foreign_call_result_from_single_value() {
        let value = FieldElement::from(42u128);
        let result = ForeignCallResult::from(value);

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], ForeignCallParam::Single(value));
    }

    #[test]
    fn test_foreign_call_result_from_vec_creates_single_array_output() {
        let values =
            vec![FieldElement::from(1u128), FieldElement::from(2u128), FieldElement::from(3u128)];
        let result = ForeignCallResult::from(values.clone());

        // From<Vec<F>> creates one output that is an Array
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], ForeignCallParam::Array(values));
    }

    #[test]
    fn test_foreign_call_result_from_params_creates_multiple_outputs() {
        let params = vec![
            ForeignCallParam::Single(FieldElement::from(1u128)),
            ForeignCallParam::Single(FieldElement::from(2u128)),
            ForeignCallParam::Single(FieldElement::from(3u128)),
        ];
        let result = ForeignCallResult::from(params.clone());

        // From<Vec<ForeignCallParam<F>>> creates multiple outputs
        assert_eq!(result.values.len(), 3);
        assert_eq!(result.values, params);
    }
}
