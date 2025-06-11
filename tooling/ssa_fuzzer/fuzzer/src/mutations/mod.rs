mod argument_mutator;
mod instruction_mutator;
mod mutator;
mod witness_mutator;

use crate::fuzz_lib::fuzz_target_lib::FuzzerData;

pub(crate) fn mutate(data: FuzzerData) -> FuzzerData {
    mutator::mutate(data)
}
