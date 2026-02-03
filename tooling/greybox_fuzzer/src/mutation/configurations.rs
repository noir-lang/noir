//! This file contains configurations for selecting particular behaviors during mutations
use rand::Rng;
use rand_xorshift::XorShiftRng;

pub struct WeightedSelectionConfig<T, const N: usize> {
    pub options_with_weights: [(T, usize); N],
    pub total_weight: usize,
}

impl<T: Copy, const N: usize> WeightedSelectionConfig<T, N> {
    pub const fn new(options_with_weights: [(T, usize); N]) -> Self {
        let mut total_weight = 0;
        let mut i = 0;
        while i < options_with_weights.len() {
            total_weight += options_with_weights[i].1;
            i += 1;
        }

        Self { options_with_weights, total_weight }
    }

    pub fn select(&self, prng: &mut XorShiftRng) -> T {
        let mut selector = prng.random_range(0..self.total_weight);
        for (option, weight) in &self.options_with_weights {
            if selector < *weight {
                return *option;
            }
            selector -= weight;
        }
        unreachable!("Should have returned by now")
    }
}

/// Mutations of individual bytes in strings
#[derive(Copy, Clone, Debug)]
pub(crate) enum ByteValueMutationOptions {
    RandomByte,
    DictionaryByte,
}
pub(crate) type ByteValueMutationConfig = WeightedSelectionConfig<ByteValueMutationOptions, 2>;
/// Splicing mutations for strings and arrays
#[derive(Copy, Clone, Debug)]
pub(crate) enum SpliceMutationOptions {
    PositionPreserving,
    RandomChunks,
}

pub(crate) type SpliceMutationConfig = WeightedSelectionConfig<SpliceMutationOptions, 2>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum UnbalancedArraySpliceOptions {
    ArraySpecific,
    Recurse,
}

pub(crate) type UnbalancedArraySpliceConfig =
    WeightedSelectionConfig<UnbalancedArraySpliceOptions, 2>;

/// Enum for splice candidate selection
#[derive(Copy, Clone, Debug)]
pub(crate) enum SpliceCandidateOptions {
    First,
    Second,
}

pub(crate) type SpliceCandidateConfig = WeightedSelectionConfig<SpliceCandidateOptions, 2>;

/// Structural mutations used in strings and arrays (arrays don't use random value)
#[derive(Copy, Clone, Debug)]
pub(crate) enum StructuralMutationOptions {
    ChaoticSelfSplice,
    ChunkDuplication,
    RandomValueDuplication,
    Swap,
}

pub(crate) type StructuralMutationConfig = WeightedSelectionConfig<StructuralMutationOptions, 4>;

/// Selection of value or structural mutation. Used in top-level input map and in strings
#[derive(Copy, Clone, Debug)]
pub(crate) enum TopLevelMutationOptions {
    Value,
    Structure,
}
pub(crate) type TopLevelMutationConfig = WeightedSelectionConfig<TopLevelMutationOptions, 2>;

// Field-specific configurations

///Field element substitution mutation types
#[derive(Copy, Clone, Debug)]
pub(crate) enum FieldElementSubstitutionMutationOptions {
    Zero,
    One,
    MinusOne,
    Dictionary,
    PowerOfTwo,
    PowerOfTwoMinusOne,
}

pub(crate) type FieldElementSubstitutionMutationConfig =
    WeightedSelectionConfig<FieldElementSubstitutionMutationOptions, 6>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum FieldElementInversionMutationOptions {
    Additive,
    Multiplicative,
}
pub(crate) type FieldElementInversionMutationConfig =
    WeightedSelectionConfig<FieldElementInversionMutationOptions, 2>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum FieldElementPow2UpdateOptions {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}
pub(crate) type FieldElementPow2UpdateConfig =
    WeightedSelectionConfig<FieldElementPow2UpdateOptions, 4>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum FieldElementSmallValueUpdateOptions {
    Addition,
    Subtraction,
    Multiplication,
}
pub(crate) type FieldElementSmallValueUpdateConfig =
    WeightedSelectionConfig<FieldElementSmallValueUpdateOptions, 3>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum FieldElementDictionaryUpdateOptions {
    Addition,
    Subtraction,
    Multiplication,
}
pub(crate) type FieldElementDictionaryUpdateConfig =
    WeightedSelectionConfig<FieldElementDictionaryUpdateOptions, 3>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum TopLevelFieldElementMutationOptions {
    Substitution,
    Inversion,
    Pow2Update,
    SmallValueUpdate,
    DictionaryUpdate,
}
pub(crate) type TopLevelFieldElementMutationConfig =
    WeightedSelectionConfig<TopLevelFieldElementMutationOptions, 5>;

/// Enum for testcase splice type
#[derive(Copy, Clone, Debug)]
pub(crate) enum TestCaseSpliceTypeOptions {
    /// Around 50% for each top-level element
    BalancedTopLevel,
    /// 80/20 for each element at lower level
    UnbalancedFull,
    /// One element merged into the main testcase
    SingleElementImport,
}

pub(crate) type TestCaseSpliceConfig = WeightedSelectionConfig<TestCaseSpliceTypeOptions, 3>;

// Int-specific configurations

#[derive(Copy, Clone, Debug)]
pub(crate) enum FixedIntSubstitutionOptions {
    Minimum,
    Maximum,
    Pow2,
}

pub(crate) type FixedIntSubstitutionConfig =
    WeightedSelectionConfig<FixedIntSubstitutionOptions, 3>;

/// Enum for binary integer operation mutation
#[derive(Copy, Clone, Debug)]
pub(crate) enum BinaryIntOperationMutationOptions {
    Add,
    Sub,
    And,
    Or,
    Xor,
}

pub(crate) type BinaryIntOperationMutationConfig =
    WeightedSelectionConfig<BinaryIntOperationMutationOptions, 5>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum IntTopLevelMutationOptions {
    FixedSubstitution,
    DictionarySubstitution,
    Negation,
    Shift,
    SmallValueUpdate,
    DictionaryValueUpdate,
    Pow2Update,
}

pub(crate) type IntTopLevelMutationConfig = WeightedSelectionConfig<IntTopLevelMutationOptions, 7>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum UnbalancedSpliceOptions {
    FirstTestCase,
    SecondTestCase,
}

pub(crate) type UnbalancedSpliceConfig = WeightedSelectionConfig<UnbalancedSpliceOptions, 2>;

pub(crate) const BASIC_UNBALANCED_VECTOR_CONFIGURATION: UnbalancedSpliceConfig =
    UnbalancedSpliceConfig::new([
        (UnbalancedSpliceOptions::FirstTestCase, 8),
        (UnbalancedSpliceOptions::SecondTestCase, 2),
    ]);
// Default configurations for all mutations that are currently used

pub(crate) const BASIC_SPLICE_MUTATION_CONFIGURATION: SpliceMutationConfig =
    SpliceMutationConfig::new([
        (SpliceMutationOptions::PositionPreserving, 1),
        (SpliceMutationOptions::RandomChunks, 1),
    ]);
pub(crate) const BASIC_UNBALANCED_ARRAY_SPLICE_MUTATION_CONFIGURATION: UnbalancedArraySpliceConfig =
    UnbalancedArraySpliceConfig::new([
        (UnbalancedArraySpliceOptions::ArraySpecific, 11),
        (UnbalancedArraySpliceOptions::Recurse, 9),
    ]);
pub(crate) const BASIC_BYTE_VALUE_MUTATION_CONFIGURATION: ByteValueMutationConfig =
    ByteValueMutationConfig::new([
        (ByteValueMutationOptions::RandomByte, 1),
        (ByteValueMutationOptions::DictionaryByte, 3),
    ]);

pub(crate) const DICTIONARY_EMPTY_BYTE_VALUE_MUTATION_CONFIGURATION: ByteValueMutationConfig =
    ByteValueMutationConfig::new([
        (ByteValueMutationOptions::RandomByte, 1),
        (ByteValueMutationOptions::DictionaryByte, 0),
    ]);

pub(crate) const BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION: SpliceCandidateConfig =
    SpliceCandidateConfig::new([
        (SpliceCandidateOptions::First, 11),
        (SpliceCandidateOptions::Second, 10),
    ]);

pub(crate) const BASIC_STRUCTURE_MUTATION_CONFIGURATION: StructuralMutationConfig =
    StructuralMutationConfig::new([
        (StructuralMutationOptions::ChaoticSelfSplice, 3),
        (StructuralMutationOptions::ChunkDuplication, 2),
        (StructuralMutationOptions::RandomValueDuplication, 1),
        (StructuralMutationOptions::Swap, 3),
    ]);

pub(crate) const BASIC_TOP_LEVEL_MUTATION_CONFIGURATION: TopLevelMutationConfig =
    TopLevelMutationConfig::new([
        (TopLevelMutationOptions::Value, 7),
        (TopLevelMutationOptions::Structure, 3),
    ]);

// Field-specific mutation configurations

pub(crate) const BASIC_FIELD_SUBSTITUTION_CONFIGURATION: FieldElementSubstitutionMutationConfig =
    FieldElementSubstitutionMutationConfig::new([
        (FieldElementSubstitutionMutationOptions::Zero, 20),
        (FieldElementSubstitutionMutationOptions::One, 20),
        (FieldElementSubstitutionMutationOptions::MinusOne, 20),
        (FieldElementSubstitutionMutationOptions::Dictionary, 20),
        (FieldElementSubstitutionMutationOptions::PowerOfTwo, 254),
        (FieldElementSubstitutionMutationOptions::PowerOfTwoMinusOne, 254),
    ]);

pub(crate) const BASIC_FIELD_INVERSION_CONFIGURATION: FieldElementInversionMutationConfig =
    FieldElementInversionMutationConfig::new([
        (FieldElementInversionMutationOptions::Additive, 10),
        (FieldElementInversionMutationOptions::Multiplicative, 1),
    ]);

pub(crate) const BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION: FieldElementPow2UpdateConfig =
    FieldElementPow2UpdateConfig::new([
        (FieldElementPow2UpdateOptions::Addition, 3),
        (FieldElementPow2UpdateOptions::Subtraction, 3),
        (FieldElementPow2UpdateOptions::Multiplication, 2),
        (FieldElementPow2UpdateOptions::Division, 1),
    ]);

pub(crate) const BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION:
    FieldElementSmallValueUpdateConfig = FieldElementSmallValueUpdateConfig::new([
    (FieldElementSmallValueUpdateOptions::Addition, 3),
    (FieldElementSmallValueUpdateOptions::Subtraction, 3),
    (FieldElementSmallValueUpdateOptions::Multiplication, 1),
]);

pub(crate) const BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION:
    FieldElementDictionaryUpdateConfig = FieldElementDictionaryUpdateConfig::new([
    (FieldElementDictionaryUpdateOptions::Addition, 2),
    (FieldElementDictionaryUpdateOptions::Subtraction, 2),
    (FieldElementDictionaryUpdateOptions::Multiplication, 1),
]);

pub(crate) const BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION:
    TopLevelFieldElementMutationConfig = TopLevelFieldElementMutationConfig::new([
    (TopLevelFieldElementMutationOptions::Substitution, 10),
    (TopLevelFieldElementMutationOptions::Inversion, 1),
    (TopLevelFieldElementMutationOptions::Pow2Update, 5),
    (TopLevelFieldElementMutationOptions::SmallValueUpdate, 10),
    (TopLevelFieldElementMutationOptions::DictionaryUpdate, 10),
]);

pub(crate) const BASIC_TESTCASE_SPLICE_CONFIGURATION: TestCaseSpliceConfig =
    TestCaseSpliceConfig::new([
        (TestCaseSpliceTypeOptions::BalancedTopLevel, 1),
        (TestCaseSpliceTypeOptions::UnbalancedFull, 1),
        (TestCaseSpliceTypeOptions::SingleElementImport, 2),
    ]);

/// Generic vector structural mutation configuration (random value duplication weight MUST stay zero)
#[allow(clippy::identity_op)]
pub(crate) const BASIC_VECTOR_STRUCTURE_MUTATION_CONFIGURATION: StructuralMutationConfig =
    StructuralMutationConfig::new([
        (StructuralMutationOptions::ChaoticSelfSplice, 3),
        (StructuralMutationOptions::ChunkDuplication, 2),
        (StructuralMutationOptions::RandomValueDuplication, 0),
        (StructuralMutationOptions::Swap, 3),
    ]);

pub(crate) const BASIC_FIXED_INT_SUBSTITUTION_CONFIGURATION: FixedIntSubstitutionConfig =
    FixedIntSubstitutionConfig::new([
        (FixedIntSubstitutionOptions::Minimum, 1),
        (FixedIntSubstitutionOptions::Maximum, 1),
        (FixedIntSubstitutionOptions::Pow2, 1),
    ]);

pub(crate) const BASIC_BINARY_INT_OPERATION_MUTATION_CONFIGURATION:
    BinaryIntOperationMutationConfig = BinaryIntOperationMutationConfig::new([
    (BinaryIntOperationMutationOptions::Add, 1),
    (BinaryIntOperationMutationOptions::Sub, 1),
    (BinaryIntOperationMutationOptions::And, 1),
    (BinaryIntOperationMutationOptions::Or, 1),
    (BinaryIntOperationMutationOptions::Xor, 1),
]);

pub(crate) const BASIC_INT_TOP_LEVEL_MUTATION_CONFIGURATION: IntTopLevelMutationConfig =
    IntTopLevelMutationConfig::new([
        (IntTopLevelMutationOptions::FixedSubstitution, 0x20),
        (IntTopLevelMutationOptions::DictionarySubstitution, 0x30),
        (IntTopLevelMutationOptions::Negation, 0x2),
        (IntTopLevelMutationOptions::Shift, 0x8),
        (IntTopLevelMutationOptions::SmallValueUpdate, 0x80),
        (IntTopLevelMutationOptions::DictionaryValueUpdate, 0x30),
        (IntTopLevelMutationOptions::Pow2Update, 0x20),
    ]);
