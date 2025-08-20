use crate::fuzz_lib::instruction::Point;
use crate::mutations::basic_types::{
    bool::{generate_random_bool, mutate_bool},
    scalar::{generate_random_scalar, mutate_scalar},
};
use crate::mutations::configuration::{
    BASIC_POINT_MUTATION_CONFIGURATION, BOOL_MUTATION_CONFIGURATION_MOSTLY_FALSE,
    BOOL_MUTATION_CONFIGURATION_MOSTLY_TRUE, GENERATE_BOOL_CONFIGURATION_MOST_FALSE,
    GENERATE_BOOL_CONFIGURATION_MOST_TRUE, PointMutationOptions,
};
use rand::rngs::StdRng;

pub(crate) fn generate_random_point(rng: &mut StdRng) -> Point {
    Point {
        scalar: generate_random_scalar(rng),
        derive_from_scalar_mul: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_TRUE),
        is_infinite: generate_random_bool(rng, GENERATE_BOOL_CONFIGURATION_MOST_FALSE),
    }
}

pub(crate) fn mutate_point(point: &mut Point, rng: &mut StdRng) {
    match BASIC_POINT_MUTATION_CONFIGURATION.select(rng) {
        PointMutationOptions::Scalar => {
            mutate_scalar(&mut point.scalar, rng);
        }
        PointMutationOptions::DeriveFromScalarMul => {
            mutate_bool(
                &mut point.derive_from_scalar_mul,
                rng,
                BOOL_MUTATION_CONFIGURATION_MOSTLY_TRUE,
            );
        }
        PointMutationOptions::IsInfinite => {
            mutate_bool(&mut point.is_infinite, rng, BOOL_MUTATION_CONFIGURATION_MOSTLY_FALSE);
        }
    }
}
