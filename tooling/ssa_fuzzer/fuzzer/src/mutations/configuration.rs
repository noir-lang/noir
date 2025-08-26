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
    ElementMutation,
    PushDefault,
}
pub(crate) type VecMutationConfig = WeightedSelectionConfig<VecMutationOptions, 6>;

pub(crate) const BASIC_VEC_MUTATION_CONFIGURATION: VecMutationConfig = VecMutationConfig::new([
    (VecMutationOptions::Random, 1),
    (VecMutationOptions::Insertion, 7),
    (VecMutationOptions::Deletion, 22),
    (VecMutationOptions::Swap, 20),
    (VecMutationOptions::ElementMutation, 100),
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

pub(crate) const BOOL_MUTATION_CONFIGURATION_MOSTLY_TRUE: BoolMutationConfig =
    BoolMutationConfig::new([(BoolMutationOptions::True, 1000), (BoolMutationOptions::False, 1)]);
pub(crate) const BOOL_MUTATION_CONFIGURATION_MOSTLY_FALSE: BoolMutationConfig =
    BoolMutationConfig::new([(BoolMutationOptions::True, 1), (BoolMutationOptions::False, 1000)]);

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

#[derive(Copy, Clone, Debug)]
pub(crate) enum BlakeHashMutationOptions {
    FieldIdx,
    LimbsCount,
}
pub(crate) type Blake2sHashMutationConfig = WeightedSelectionConfig<BlakeHashMutationOptions, 2>;
pub(crate) const BASIC_BLAKE_HASH_MUTATION_CONFIGURATION: Blake2sHashMutationConfig =
    Blake2sHashMutationConfig::new([
        (BlakeHashMutationOptions::FieldIdx, 1),
        (BlakeHashMutationOptions::LimbsCount, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum Sha256CompressionMutationOptions {
    InputIndices,
    StateIndices,
    LoadElementsOfArray,
}
pub(crate) type Sha256CompressionMutationConfig =
    WeightedSelectionConfig<Sha256CompressionMutationOptions, 3>;
pub(crate) const BASIC_SHA256_COMPRESSION_MUTATION_CONFIGURATION: Sha256CompressionMutationConfig =
    Sha256CompressionMutationConfig::new([
        (Sha256CompressionMutationOptions::InputIndices, 1),
        (Sha256CompressionMutationOptions::StateIndices, 1),
        (Sha256CompressionMutationOptions::LoadElementsOfArray, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum Aes128EncryptMutationOptions {
    InputIdx,
    InputLimbsCount,
    KeyIdx,
    IvIdx,
}
pub(crate) type Aes128EncryptMutationConfig =
    WeightedSelectionConfig<Aes128EncryptMutationOptions, 4>;
pub(crate) const BASIC_AES_128_ENCRYPT_MUTATION_CONFIGURATION: Aes128EncryptMutationConfig =
    Aes128EncryptMutationConfig::new([
        (Aes128EncryptMutationOptions::InputIdx, 1),
        (Aes128EncryptMutationOptions::InputLimbsCount, 1),
        (Aes128EncryptMutationOptions::KeyIdx, 1),
        (Aes128EncryptMutationOptions::IvIdx, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum ScalarMutationOptions {
    FieldLoIdx,
    FieldHiIdx,
}
pub(crate) type ScalarMutationConfig = WeightedSelectionConfig<ScalarMutationOptions, 2>;
pub(crate) const BASIC_SCALAR_MUTATION_CONFIGURATION: ScalarMutationConfig =
    ScalarMutationConfig::new([
        (ScalarMutationOptions::FieldLoIdx, 1),
        (ScalarMutationOptions::FieldHiIdx, 1),
    ]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum PointMutationOptions {
    Scalar,
    DeriveFromScalarMul,
    IsInfinite,
}
pub(crate) type PointMutationConfig = WeightedSelectionConfig<PointMutationOptions, 3>;
pub(crate) const BASIC_POINT_MUTATION_CONFIGURATION: PointMutationConfig =
    PointMutationConfig::new([
        (PointMutationOptions::Scalar, 1),
        (PointMutationOptions::DeriveFromScalarMul, 1),
        (PointMutationOptions::IsInfinite, 1),
    ]);

// =================== GENERATION CONFIGURATIONS ==================
#[derive(Copy, Clone, Debug)]
pub(crate) enum GenerateBool {
    True,
    False,
}
pub(crate) type GenerateBoolConfig = WeightedSelectionConfig<GenerateBool, 2>;
pub(crate) const BASIC_GENERATE_BOOL_CONFIGURATION: GenerateBoolConfig =
    GenerateBoolConfig::new([(GenerateBool::True, 1), (GenerateBool::False, 1)]);
pub(crate) const GENERATE_BOOL_CONFIGURATION_MOST_TRUE: GenerateBoolConfig =
    GenerateBoolConfig::new([(GenerateBool::True, 999), (GenerateBool::False, 1)]);
pub(crate) const GENERATE_BOOL_CONFIGURATION_MOST_FALSE: GenerateBoolConfig =
    GenerateBoolConfig::new([(GenerateBool::True, 1), (GenerateBool::False, 999)]);

#[derive(Copy, Clone, Debug)]
pub(crate) enum GenerateValueType {
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
pub(crate) type GenerateValueTypeConfig = WeightedSelectionConfig<GenerateValueType, 11>;
pub(crate) const BASIC_GENERATE_VALUE_TYPE_CONFIGURATION: GenerateValueTypeConfig =
    GenerateValueTypeConfig::new([
        (GenerateValueType::Field, 1),
        (GenerateValueType::Boolean, 1),
        (GenerateValueType::U8, 1),
        (GenerateValueType::U16, 1),
        (GenerateValueType::U32, 1),
        (GenerateValueType::U64, 1),
        (GenerateValueType::U128, 1),
        (GenerateValueType::I8, 1),
        (GenerateValueType::I16, 1),
        (GenerateValueType::I32, 1),
        (GenerateValueType::I64, 1),
    ]);

// Compile-time check that configuration has correct number of entries
const _: () = {
    use noir_ssa_fuzzer::typed_value::ValueType;
    use strum::EnumCount;
    assert!(
        BASIC_GENERATE_VALUE_TYPE_CONFIGURATION.options_with_weights.len() == ValueType::COUNT,
        "BASIC_GENERATE_VALUE_TYPE_CONFIGURATION must have an entry for every GenerateValueType variant"
    );
};

#[derive(Copy, Clone, Debug)]
pub(crate) enum GenerateCommand {
    InsertJmpIfBlock,
    InsertJmpBlock,
    InsertCycle,
    InsertFunctionCall,
    InsertSimpleInstructionBlock,
    SwitchToNextBlock,
}
pub(crate) type GenerateCommandConfig = WeightedSelectionConfig<GenerateCommand, 6>;
pub(crate) const BASIC_GENERATE_COMMAND_CONFIGURATION: GenerateCommandConfig =
    GenerateCommandConfig::new([
        (GenerateCommand::InsertJmpIfBlock, 1),
        (GenerateCommand::InsertJmpBlock, 1),
        (GenerateCommand::InsertCycle, 1),
        (GenerateCommand::InsertFunctionCall, 1),
        (GenerateCommand::InsertSimpleInstructionBlock, 1),
        (GenerateCommand::SwitchToNextBlock, 1),
    ]);
// Compile-time check that configuration has correct number of entries
const _: () = {
    use crate::fuzz_lib::function_context::FuzzerFunctionCommand;
    use strum::EnumCount;
    assert!(
        BASIC_GENERATE_COMMAND_CONFIGURATION.options_with_weights.len()
            == FuzzerFunctionCommand::COUNT,
        "BASIC_GENERATE_COMMAND_CONFIGURATION must have an entry for every FuzzerFunctionCommand variant"
    );
};

#[derive(Copy, Clone, Debug)]
pub(crate) enum GenerateInstruction {
    AddChecked,
    SubChecked,
    MulChecked,
    Div,
    Eq,
    Mod,
    Not,
    Shl,
    Shr,
    Cast,
    And,
    Or,
    Xor,
    Lt,
    AddSubConstrain,
    MulDivConstrain,
    AddToMemory,
    LoadFromMemory,
    SetToMemory,
    CreateArray,
    ArrayGet,
    ArraySet,
    ArrayGetWithConstantIndex,
    ArraySetWithConstantIndex,
    FieldToBytesToField,
    Blake2sHash,
    Blake3Hash,
    Keccakf1600Hash,
    Aes128Encrypt,
    Sha256Compression,
    PointAdd,
    MultiScalarMul,
    EcdsaSecp256r1,
    EcdsaSecp256k1,
}
pub(crate) type GenerateInstructionConfig = WeightedSelectionConfig<GenerateInstruction, 34>;
pub(crate) const BASIC_GENERATE_INSTRUCTION_CONFIGURATION: GenerateInstructionConfig =
    GenerateInstructionConfig::new([
        (GenerateInstruction::AddChecked, 100),
        (GenerateInstruction::SubChecked, 100),
        (GenerateInstruction::MulChecked, 100),
        (GenerateInstruction::Div, 100),
        (GenerateInstruction::Eq, 100),
        (GenerateInstruction::Mod, 100),
        (GenerateInstruction::Not, 100),
        (GenerateInstruction::Shl, 100),
        (GenerateInstruction::Shr, 100),
        (GenerateInstruction::Cast, 100),
        (GenerateInstruction::And, 100),
        (GenerateInstruction::Or, 100),
        (GenerateInstruction::Xor, 100),
        (GenerateInstruction::Lt, 100),
        (GenerateInstruction::AddSubConstrain, 100),
        (GenerateInstruction::MulDivConstrain, 100),
        (GenerateInstruction::AddToMemory, 100),
        (GenerateInstruction::LoadFromMemory, 100),
        (GenerateInstruction::SetToMemory, 100),
        (GenerateInstruction::CreateArray, 100),
        (GenerateInstruction::ArrayGet, 100),
        (GenerateInstruction::ArraySet, 100),
        (GenerateInstruction::ArrayGetWithConstantIndex, 100),
        (GenerateInstruction::ArraySetWithConstantIndex, 100),
        // generating this instruction with smaller probability, because they are too heavy
        (GenerateInstruction::FieldToBytesToField, 5),
        (GenerateInstruction::Blake2sHash, 5),
        (GenerateInstruction::Blake3Hash, 5),
        (GenerateInstruction::Keccakf1600Hash, 5),
        (GenerateInstruction::Aes128Encrypt, 5),
        (GenerateInstruction::Sha256Compression, 5),
        (GenerateInstruction::PointAdd, 5),
        (GenerateInstruction::MultiScalarMul, 5),
        (GenerateInstruction::EcdsaSecp256r1, 5),
        (GenerateInstruction::EcdsaSecp256k1, 5),
    ]);

// Compile-time check that configuration has correct number of entries
const _: () = {
    use crate::fuzz_lib::instruction::Instruction;
    use strum::EnumCount;
    assert!(
        BASIC_GENERATE_INSTRUCTION_CONFIGURATION.options_with_weights.len() == Instruction::COUNT,
        "BASIC_GENERATE_INSTRUCTION_CONFIGURATION must have an entry for every Instruction variant"
    );
};
