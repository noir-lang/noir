use crate::fuzz_lib::base_context::FuzzerCommand;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait MutateVecFuzzerCommand {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand>;
}

trait MutateVecFuzzerCommandFactory {
    fn new() -> Box<dyn MutateVecFuzzerCommand>;
}

struct RandomMutation;
impl MutateVecFuzzerCommand for RandomMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut bytes = [0u8; 128];
        rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }
}
impl MutateVecFuzzerCommandFactory for RandomMutation {
    fn new() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(RandomMutation)
    }
}
// todo more mutations
fn mutation_factory(rng: &mut StdRng) -> Box<dyn MutateVecFuzzerCommand> {
    let mutator = if rng.gen_bool(0.5) { RandomMutation::new() } else { RandomMutation::new() };
    mutator
}

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: Vec<FuzzerCommand>,
    rng: &mut StdRng,
) -> Vec<FuzzerCommand> {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, vec_fuzzer_command)
}
