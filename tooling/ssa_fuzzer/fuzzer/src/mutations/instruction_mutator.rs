use crate::fuzz_lib::instruction::Instruction;
use crate::mutations::argument_mutator::argument_mutator;
use libfuzzer_sys::arbitrary::Unstructured;
use rand::{Rng, rngs::StdRng};

trait InstructionMutatorStrategy<'a> {
    fn mutate(&mut self, value: Instruction) -> Instruction;
    fn new(rng: &'a mut StdRng) -> Self;
}

struct RandomMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> InstructionMutatorStrategy<'a> for RandomMutation<'a> {
    fn mutate(&mut self, value: Instruction) -> Instruction {
        let mut bytes = [0u8; 17];
        self.rng.fill(&mut bytes);
        Unstructured::new(&bytes).arbitrary().unwrap()
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct InstructionArgumentsMutation<'a> {
    rng: &'a mut StdRng,
}
impl<'a> InstructionMutatorStrategy<'a> for InstructionArgumentsMutation<'a> {
    fn mutate(&mut self, value: Instruction) -> Instruction {
        match value {
            Instruction::AddChecked { lhs, rhs } => Instruction::AddChecked {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::SubChecked { lhs, rhs } => Instruction::SubChecked {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::MulChecked { lhs, rhs } => Instruction::MulChecked {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Div { lhs, rhs } => Instruction::Div {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Eq { lhs, rhs } => Instruction::Eq {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Mod { lhs, rhs } => Instruction::Mod {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Not { lhs } => Instruction::Not { lhs: argument_mutator(lhs, self.rng) },
            Instruction::Shl { lhs, rhs } => Instruction::Shl {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Shr { lhs, rhs } => Instruction::Shr {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Cast { lhs, type_ } => {
                Instruction::Cast { lhs: argument_mutator(lhs, self.rng), type_ }
            }
            Instruction::And { lhs, rhs } => Instruction::And {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Or { lhs, rhs } => Instruction::Or {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Xor { lhs, rhs } => Instruction::Xor {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::Lt { lhs, rhs } => Instruction::Lt {
                lhs: argument_mutator(lhs, self.rng),
                rhs: argument_mutator(rhs, self.rng),
            },
            Instruction::AddSubConstrain { lhs, rhs } => Instruction::AddSubConstrain { lhs, rhs },
            Instruction::MulDivConstrain { lhs, rhs } => Instruction::MulDivConstrain { lhs, rhs },
            Instruction::AddToMemory { lhs } => {
                Instruction::AddToMemory { lhs: argument_mutator(lhs, self.rng) }
            }
            Instruction::LoadFromMemory { memory_addr } => {
                Instruction::LoadFromMemory { memory_addr: argument_mutator(memory_addr, self.rng) }
            }
            Instruction::SetToMemory { memory_addr_index, value } => Instruction::SetToMemory {
                memory_addr_index,
                value: argument_mutator(value, self.rng),
            },
        }
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

struct DefaultMutationStrategy<'a> {
    rng: &'a mut StdRng,
}
impl<'a> InstructionMutatorStrategy<'a> for DefaultMutationStrategy<'a> {
    fn mutate(&mut self, value: Instruction) -> Instruction {
        value
    }

    fn new(rng: &'a mut StdRng) -> Self {
        Self { rng }
    }
}

pub(crate) fn instruction_mutator(instruction: Instruction, rng: &mut StdRng) -> Instruction {
    let instruction = if rng.gen_bool(0.5) {
        RandomMutation::new(rng).mutate(instruction)
    } else if rng.gen_bool(0.3) {
        InstructionArgumentsMutation::new(rng).mutate(instruction)
    } else {
        DefaultMutationStrategy::new(rng).mutate(instruction)
    };
    instruction
}
