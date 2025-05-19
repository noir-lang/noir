use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::typed_value::ValueType;

#[derive(Arbitrary, Debug, Clone, Copy)]
pub(crate) struct Argument {
    /// Index of the argument in the context of stored variables of this type
    /// e.g. if we have variables with ids [0, 1] in u64 vector and variables with ids [5, 8] in fields vector
    /// Argument(Index(0), ValueType::U64) -> id 0
    /// Argument(Index(0), ValueType::Field) -> id 5
    /// Argument(Index(1), ValueType::Field) -> id 8
    pub(crate) index: usize,
    /// Type of the argument
    pub(crate) value_type: ValueType,
}

/// TODO: For operations that take two arguments we ignore type of the second argument.
#[derive(Arbitrary, Debug, Clone, Copy)]
pub(crate) enum Instruction {
    /// Addition of two values
    AddChecked {
        lhs: Argument,
        rhs: Argument,
    },
    /// Subtraction of two values
    SubChecked {
        lhs: Argument,
        rhs: Argument,
    },
    /// Multiplication of two values
    MulChecked {
        lhs: Argument,
        rhs: Argument,
    },
    /// Division of two values
    Div {
        lhs: Argument,
        rhs: Argument,
    },
    /// Equality comparison
    Eq {
        lhs: Argument,
        rhs: Argument,
    },
    /// Modulo operation
    Mod {
        lhs: Argument,
        rhs: Argument,
    },
    /// Bitwise NOT
    Not {
        lhs: Argument,
    },
    /// Left shift
    Shl {
        lhs: Argument,
        rhs: Argument,
    },
    /// Right shift
    Shr {
        lhs: Argument,
        rhs: Argument,
    },
    /// Cast into type
    Cast {
        lhs: Argument,
        type_: ValueType,
    },
    /// Bitwise AND
    And {
        lhs: Argument,
        rhs: Argument,
    },
    /// Bitwise OR
    Or {
        lhs: Argument,
        rhs: Argument,
    },
    /// Bitwise XOR
    Xor {
        lhs: Argument,
        rhs: Argument,
    },

    /// Less than comparison
    Lt {
        lhs: Argument,
        rhs: Argument,
    },

    /// constrain(lhs == lhs + rhs - rhs), doesn't insert constraint if idempotent_morphing_enabled=false
    /// uses only fields variables
    AddSubConstrain {
        lhs: usize,
        rhs: usize,
    },
    /// constrain(lhs == lhs * rhs / rhs), doesn't insert constraint if idempotent_morphing_enabled=false
    /// uses only fields variables
    MulDivConstrain {
        lhs: usize,
        rhs: usize,
    },

    AddToMemory {
        lhs: Argument,
    },
    LoadFromMemory {
        memory_addr: Argument,
    },
    SetToMemory {
        memory_addr_index: usize,
        value: Argument,
    },
}

/// Check if two vectors of arguments have the same types
/// lhs and rhs are the same length, its inputs for the same instruction
/// we need to check if all arguments have same types up to permutation
fn have_same_types(lhs: Vec<Argument>, rhs: Vec<Argument>) -> bool {
    if lhs.len() == 1 {
        return lhs[0].value_type == rhs[0].value_type;
    } else if lhs.len() == 2 {
        return (lhs[0].value_type == rhs[0].value_type && lhs[1].value_type == rhs[1].value_type)
            || (lhs[0].value_type == rhs[1].value_type && lhs[1].value_type == rhs[0].value_type);
    } else {
        unreachable!()
    }
}

impl Instruction {
    fn get_arguments(&self) -> Vec<Argument> {
        match self {
            Instruction::AddChecked { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::SubChecked { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::MulChecked { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Div { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Eq { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Mod { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Not { lhs } => vec![*lhs],
            Instruction::Shl { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Shr { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::And { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Or { lhs, rhs } => vec![*lhs, *rhs],
            Instruction::Xor { lhs, rhs } => vec![*lhs, *rhs],
            _ => unreachable!(),
        }
    }
}

/// Implement PartialEq for Instruction to allow for comparison of instructions
/// to forbid certain instructions
impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Instruction::Cast { lhs: arg1, type_: l_type },
                Instruction::Cast { lhs: arg2, type_: r_type },
            ) => l_type == r_type && have_same_types(vec![*arg1], vec![*arg2]),
            _ => {
                std::mem::discriminant(self) == std::mem::discriminant(other)
                    && have_same_types(self.get_arguments(), other.get_arguments())
            }
        }
    }
}

/// Represents set of instructions
/// NOT EQUAL TO SSA BLOCK
#[derive(Arbitrary, Debug, Clone)]
pub(crate) struct InstructionBlock {
    pub(crate) instructions: Vec<Instruction>,
}
