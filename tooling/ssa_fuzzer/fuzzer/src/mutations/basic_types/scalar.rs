use crate::fuzz_lib::instruction::Scalar;
use crate::mutations::configuration::{BASIC_SCALAR_MUTATION_CONFIGURATION, ScalarMutationOptions};
use rand::{Rng, rngs::StdRng};

pub(crate) fn generate_random_scalar(rng: &mut StdRng) -> Scalar {
    Scalar {
        field_lo_idx: rng.random_range(usize::MIN..usize::MAX),
        field_hi_idx: rng.random_range(usize::MIN..usize::MAX),
    }
}

pub(crate) fn mutate_scalar(scalar: &mut Scalar, rng: &mut StdRng) {
    match BASIC_SCALAR_MUTATION_CONFIGURATION.select(rng) {
        ScalarMutationOptions::FieldLoIdx => {
            scalar.field_lo_idx = rng.random_range(usize::MIN..usize::MAX);
        }
        ScalarMutationOptions::FieldHiIdx => {
            scalar.field_hi_idx = rng.random_range(usize::MIN..usize::MAX);
        }
    }
}
