use dep::types::traits::Empty;

pub fn validate_value_transformation<T, S, N, Env>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool
) {
    for i in 0..N {
        assert(is_transformed(original_array[i], transformed_value_array[i]), "invalid transformed value");
    }
}

pub fn validate_transformed_values<T, S, N, Env>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    transform_value: fn[Env](T) -> S
) where S: Empty + Eq {
    validate_value_transformation(
        original_array,
        transformed_value_array,
        |original: T, transformed: S| transformed == transform_value(original)
    );
}
