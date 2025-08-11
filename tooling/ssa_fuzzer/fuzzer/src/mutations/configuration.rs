//! This file contains configurations for selecting particular behaviors during mutations
use rand::{Rng, rngs::StdRng};

pub(crate) const MAX_NUMBER_OF_MUTATIONS: usize = 25;
pub(crate) const SIZE_OF_SMALL_ARBITRARY_BUFFER: usize = 25;
pub(crate) const SIZE_OF_LARGE_ARBITRARY_BUFFER: usize = 1024;

pub(crate) struct WeightedSelectionConfig<T, const N: usize> {
    pub(crate) options_with_weights: [(T, usize); N],
    pub(crate) total_weight: usize,
}

impl<T: Copy, const N: usize> WeightedSelectionConfig<T, N> {
    pub(crate) const fn new(options_with_weights: [(T, usize); N]) -> Self {
        let mut total_weight = 0;
        let mut i = 0;
        while i < options_with_weights.len() {
            total_weight += options_with_weights[i].1;
            i += 1;
        }

        Self { options_with_weights, total_weight }
    }

    pub(crate) fn select(&self, rng: &mut StdRng) -> T {
        let mut selector = rng.gen_range(0..self.total_weight);
        for (option, weight) in &self.options_with_weights {
            if selector < *weight {
                return *option;
            }
            selector -= weight;
        }
        unreachable!("Should have returned by now")
    }
}

/// Mutations config for single FuzzerData mutations
#[derive(Copy, Clone, Debug)]
pub(crate) enum FuzzerDataMutationOptions {
    Functions,
    InstructionBlocks,
    Witnesses,
}
pub(crate) type FuzzerDataMutationConfig = WeightedSelectionConfig<FuzzerDataMutationOptions, 3>;
pub(crate) const BASIC_FUZZER_DATA_MUTATION_CONFIGURATION: FuzzerDataMutationConfig =
    FuzzerDataMutationConfig::new([
        (FuzzerDataMutationOptions::Functions, 1),
        (FuzzerDataMutationOptions::InstructionBlocks, 1),
        (FuzzerDataMutationOptions::Witnesses, 4),
    ]);

/// Mutations config for function mutations
#[derive(Copy, Clone, Debug)]
pub(crate) enum FunctionMutationOptions {
    ReturnBlockIdx,
    FunctionFuzzerCommands,
    ReturnType,
}

pub(crate) type MutationConfig = WeightedSelectionConfig<FunctionMutationOptions, 3>;
pub(crate) const BASIC_FUNCTION_MUTATION_CONFIGURATION: MutationConfig = MutationConfig::new([
    (FunctionMutationOptions::ReturnBlockIdx, 1),
    (FunctionMutationOptions::FunctionFuzzerCommands, 1),
    (FunctionMutationOptions::ReturnType, 1),
]);

/// Mutations of witness values
#[derive(Copy, Clone, Debug)]
pub(crate) enum WitnessMutationOptions {
    Random,
    MaxValue,
    MinValue,
    SmallAddSub,
    PowerOfTwoAddSub,
}

pub(crate) type WitnessMutationConfig = WeightedSelectionConfig<WitnessMutationOptions, 5>;
pub(crate) const BASIC_WITNESS_MUTATION_CONFIGURATION: WitnessMutationConfig =
    WitnessMutationConfig::new([
        (WitnessMutationOptions::Random, 1),
        (WitnessMutationOptions::MaxValue, 2),
        (WitnessMutationOptions::MinValue, 2),
        (WitnessMutationOptions::SmallAddSub, 4),
        (WitnessMutationOptions::PowerOfTwoAddSub, 3),
    ]);

/// Mutations of instructions
#[derive(Copy, Clone, Debug)]
pub(crate) enum InstructionMutationOptions {
    Random,
    ArgumentMutation,
}

pub(crate) type InstructionMutationConfig = WeightedSelectionConfig<InstructionMutationOptions, 2>;
pub(crate) const BASIC_INSTRUCTION_MUTATION_CONFIGURATION: InstructionMutationConfig =
    InstructionMutationConfig::new([
        (InstructionMutationOptions::Random, 1),
        (InstructionMutationOptions::ArgumentMutation, 4),
    ]);

/// Instruction argument mutation configuration
#[derive(Copy, Clone, Debug)]
pub(crate) enum InstructionArgumentMutationOptions {
    Left,
    Right,
}

pub(crate) type InstructionArgumentMutationConfig =
    WeightedSelectionConfig<InstructionArgumentMutationOptions, 2>;
pub(crate) const BASIC_INSTRUCTION_ARGUMENT_MUTATION_CONFIGURATION:
    InstructionArgumentMutationConfig = InstructionArgumentMutationConfig::new([
    // Fuzzer uses type of the left variable for binary ops,
    // so mutating the right variables makes less sense
    (InstructionArgumentMutationOptions::Left, 5),
    (InstructionArgumentMutationOptions::Right, 1),
]);

/// Mutations of arguments of instructions
#[derive(Copy, Clone, Debug)]
pub(crate) enum ArgumentMutationOptions {
    MutateIndex,
    ChangeType,
}

pub(crate) type ArgumentMutationConfig = WeightedSelectionConfig<ArgumentMutationOptions, 2>;
pub(crate) const BASIC_ARGUMENT_MUTATION_CONFIGURATION: ArgumentMutationConfig =
    ArgumentMutationConfig::new([
        (ArgumentMutationOptions::MutateIndex, 7),
        (ArgumentMutationOptions::ChangeType, 2),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum VecMutationOptions {
    Random,
    Insertion,
    Deletion,
    Swap,
    Mutation,
    PushDefault,
}
pub(crate) type VecMutationConfig = WeightedSelectionConfig<VecMutationOptions, 6>;

pub(crate) const BASIC_VEC_MUTATION_CONFIGURATION: VecMutationConfig = VecMutationConfig::new([
    (VecMutationOptions::Random, 1),
    (VecMutationOptions::Insertion, 7),
    (VecMutationOptions::Deletion, 22),
    (VecMutationOptions::Swap, 20),
    (VecMutationOptions::Mutation, 100),
    (VecMutationOptions::PushDefault, 15),
]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum ValueTypeMutationOptions {
    Field,
    Boolean,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
}
pub(crate) type ValueTypeMutationConfig = WeightedSelectionConfig<ValueTypeMutationOptions, 11>;
pub(crate) const BASIC_VALUE_TYPE_MUTATION_CONFIGURATION: ValueTypeMutationConfig =
    ValueTypeMutationConfig::new([
        (ValueTypeMutationOptions::Field, 1),
        (ValueTypeMutationOptions::Boolean, 1),
        (ValueTypeMutationOptions::U8, 1),
        (ValueTypeMutationOptions::U16, 1),
        (ValueTypeMutationOptions::U32, 1),
        (ValueTypeMutationOptions::U64, 1),
        (ValueTypeMutationOptions::U128, 1),
        (ValueTypeMutationOptions::I8, 1),
        (ValueTypeMutationOptions::I16, 1),
        (ValueTypeMutationOptions::I32, 1),
        (ValueTypeMutationOptions::I64, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum BoolMutationOptions {
    True,
    False,
}
pub(crate) type BoolMutationConfig = WeightedSelectionConfig<BoolMutationOptions, 2>;
pub(crate) const BASIC_BOOL_MUTATION_CONFIGURATION: BoolMutationConfig =
    BoolMutationConfig::new([(BoolMutationOptions::True, 1), (BoolMutationOptions::False, 1)]);

pub(crate) const BASIC_SAFE_INDEX_MUTATION_CONFIGURATION: BoolMutationConfig =
    BoolMutationConfig::new([(BoolMutationOptions::True, 1000), (BoolMutationOptions::False, 1)]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum UsizeMutationOptions {
    Random,
    Increment,
    Decrement,
    AddRandom,
    SubtractRandom,
}
pub(crate) type UsizeMutationConfig = WeightedSelectionConfig<UsizeMutationOptions, 5>;
pub(crate) const BASIC_USIZE_MUTATION_CONFIGURATION: UsizeMutationConfig =
    UsizeMutationConfig::new([
        (UsizeMutationOptions::Random, 1),
        (UsizeMutationOptions::Increment, 1),
        (UsizeMutationOptions::Decrement, 1),
        (UsizeMutationOptions::AddRandom, 1),
        (UsizeMutationOptions::SubtractRandom, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum ArrayGetMutationOptions {
    ArrayIndex,
    Index,
    SafeIndex,
}
pub(crate) type ArrayGetMutationConfig = WeightedSelectionConfig<ArrayGetMutationOptions, 3>;
pub(crate) const BASIC_ARRAY_GET_MUTATION_CONFIGURATION: ArrayGetMutationConfig =
    ArrayGetMutationConfig::new([
        (ArrayGetMutationOptions::ArrayIndex, 5),
        (ArrayGetMutationOptions::Index, 5),
        (ArrayGetMutationOptions::SafeIndex, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum ArraySetMutationOptions {
    ArrayIndex,
    Index,
    ValueIndex,
    SafeIndex,
}
pub(crate) type ArraySetMutationConfig = WeightedSelectionConfig<ArraySetMutationOptions, 4>;
pub(crate) const BASIC_ARRAY_SET_MUTATION_CONFIGURATION: ArraySetMutationConfig =
    ArraySetMutationConfig::new([
        (ArraySetMutationOptions::ArrayIndex, 5),
        (ArraySetMutationOptions::Index, 5),
        (ArraySetMutationOptions::ValueIndex, 5),
        (ArraySetMutationOptions::SafeIndex, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum CreateArrayMutationOptions {
    ElementsIndices,
    ElementType,
    IsReferences,
}
pub(crate) type CreateArrayMutationConfig = WeightedSelectionConfig<CreateArrayMutationOptions, 3>;
pub(crate) const BASIC_CREATE_ARRAY_MUTATION_CONFIGURATION: CreateArrayMutationConfig =
    CreateArrayMutationConfig::new([
        (CreateArrayMutationOptions::ElementsIndices, 2),
        (CreateArrayMutationOptions::ElementType, 2),
        (CreateArrayMutationOptions::IsReferences, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum InsertJmpIfBlockMutationOptions {
    BlockThenIdx,
    BlockElseIdx,
}
pub(crate) type InsertJmpIfBlockMutationConfig =
    WeightedSelectionConfig<InsertJmpIfBlockMutationOptions, 2>;
pub(crate) const BASIC_INSERT_JMP_IF_BLOCK_MUTATION_CONFIGURATION: InsertJmpIfBlockMutationConfig =
    InsertJmpIfBlockMutationConfig::new([
        (InsertJmpIfBlockMutationOptions::BlockThenIdx, 1),
        (InsertJmpIfBlockMutationOptions::BlockElseIdx, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum InsertCycleMutationOptions {
    BlockBodyIdx,
    StartIter,
    EndIter,
}
pub(crate) type InsertCycleMutationConfig = WeightedSelectionConfig<InsertCycleMutationOptions, 3>;
pub(crate) const BASIC_INSERT_CYCLE_MUTATION_CONFIGURATION: InsertCycleMutationConfig =
    InsertCycleMutationConfig::new([
        (InsertCycleMutationOptions::BlockBodyIdx, 1),
        (InsertCycleMutationOptions::StartIter, 1),
        (InsertCycleMutationOptions::EndIter, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum InsertFunctionCallMutationOptions {
    FunctionIdx,
    Args,
}
pub(crate) type InsertFunctionCallMutationConfig =
    WeightedSelectionConfig<InsertFunctionCallMutationOptions, 2>;
pub(crate) const BASIC_INSERT_FUNCTION_CALL_MUTATION_CONFIGURATION:
    InsertFunctionCallMutationConfig = InsertFunctionCallMutationConfig::new([
    (InsertFunctionCallMutationOptions::FunctionIdx, 1),
    (InsertFunctionCallMutationOptions::Args, 7),
]);
