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

struct RemoveCommandMutation;
impl MutateVecFuzzerCommand for RemoveCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value;
        if commands.len() > 0 {
            commands.remove(rng.gen_range(0..commands.len()));
        }
        commands
    }
}
impl MutateVecFuzzerCommandFactory for RemoveCommandMutation {
    fn new() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(RemoveCommandMutation)
    }
}

struct AddCommandMutation;
impl MutateVecFuzzerCommand for AddCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value.clone();
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        commands.push(command);
        commands
    }
}
impl MutateVecFuzzerCommandFactory for AddCommandMutation {
    fn new() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(AddCommandMutation)
    }
}

struct ReplaceCommandMutation;
impl MutateVecFuzzerCommand for ReplaceCommandMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        let mut commands = value;
        let mut bytes = [0u8; 25];
        rng.fill(&mut bytes);
        let command = Unstructured::new(&bytes).arbitrary().unwrap();
        if commands.len() > 0 {
            let command_idx = rng.gen_range(0..commands.len());
            commands[command_idx] = command;
        }
        commands
    }
}
impl MutateVecFuzzerCommandFactory for ReplaceCommandMutation {
    fn new() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(ReplaceCommandMutation)
    }
}

struct DefaultMutation;
impl MutateVecFuzzerCommand for DefaultMutation {
    fn mutate(&self, rng: &mut StdRng, value: Vec<FuzzerCommand>) -> Vec<FuzzerCommand> {
        value
    }
}
impl MutateVecFuzzerCommandFactory for DefaultMutation {
    fn new() -> Box<dyn MutateVecFuzzerCommand> {
        Box::new(DefaultMutation)
    }
}

// todo more mutations
fn mutation_factory(rng: &mut StdRng) -> Box<dyn MutateVecFuzzerCommand> {
    let mutator = if rng.gen_bool(0.5) {
        RandomMutation::new()
    } else if rng.gen_bool(0.3) {
        RemoveCommandMutation::new()
    } else if rng.gen_bool(0.2) {
        AddCommandMutation::new()
    } else if rng.gen_bool(0.1) {
        ReplaceCommandMutation::new()
    } else {
        DefaultMutation::new()
    };
    mutator
}

pub(crate) fn mutate_vec_fuzzer_command(
    vec_fuzzer_command: Vec<FuzzerCommand>,
    rng: &mut StdRng,
) -> Vec<FuzzerCommand> {
    let mutator = mutation_factory(rng);
    mutator.mutate(rng, vec_fuzzer_command)
}
