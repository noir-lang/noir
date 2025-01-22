use rand::Rng;
use rand_xorshift::XorShiftRng;

/// This file contains configurations for selecting particular behaviors during mutations
///

/// Mutations of individual bytes in strings
pub(crate) enum ByteValueMutation {
    RandomByte,
    DictionaryByte,
}

pub(crate) struct ByteValueMutationConfiguration {
    random_byte_mutation_weight: usize,
    #[allow(unused)]
    dictionary_byte_mutation_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting a byte value mutation
impl ByteValueMutationConfiguration {
    #[allow(unused)]
    pub fn new(random_byte_mutation_weight: usize, dictionary_byte_mutation_weight: usize) -> Self {
        let total_weight = random_byte_mutation_weight + dictionary_byte_mutation_weight;
        Self { random_byte_mutation_weight, dictionary_byte_mutation_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> ByteValueMutation {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.random_byte_mutation_weight {
            return ByteValueMutation::RandomByte;
        }
        return ByteValueMutation::DictionaryByte;
    }
}

/// Splicing mutations for strings and arrays
pub(crate) enum SpliceMutation {
    PositionPreserving,
    RandomChunks,
}

pub(crate) struct SpliceMutationConfiguration {
    position_preserving_mutation_weight: usize,
    #[allow(unused)]
    random_chunks_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting the splicing mechanism
impl SpliceMutationConfiguration {
    #[allow(unused)]
    pub fn new(position_preserving_mutation_weight: usize, random_chunks_weight: usize) -> Self {
        let total_weight = position_preserving_mutation_weight + random_chunks_weight;
        Self { position_preserving_mutation_weight, random_chunks_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> SpliceMutation {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.position_preserving_mutation_weight {
            return SpliceMutation::PositionPreserving;
        }
        return SpliceMutation::RandomChunks;
    }
}

pub(crate) enum UnbalancedArraySpliceType {
    ArraySpecific,
    Recurse,
}

pub(crate) struct UnbalancedArraySpliceConfiguration {
    array_specific_weight: usize,
    #[allow(unused)]
    recurse_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting the splicing mechanism
impl UnbalancedArraySpliceConfiguration {
    #[allow(unused)]
    pub fn new(array_specific_weight: usize, recurse_weight: usize) -> Self {
        let total_weight = array_specific_weight + recurse_weight;
        Self { array_specific_weight, recurse_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> UnbalancedArraySpliceType {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.array_specific_weight {
            return UnbalancedArraySpliceType::ArraySpecific;
        }
        return UnbalancedArraySpliceType::Recurse;
    }
}
/// Enum for splice candidate selection
pub(crate) enum SpliceCandidate {
    First,
    Second,
}

pub(crate) struct SpliceCandidatePrioritizationConfiguration {
    first_weight: usize,
    #[allow(unused)]
    second_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting which candidate to use for a spliced chunk
impl SpliceCandidatePrioritizationConfiguration {
    #[allow(unused)]
    pub fn new(first_weight: usize, second_weight: usize) -> Self {
        let total_weight = first_weight + second_weight;
        Self { first_weight, second_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> SpliceCandidate {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.first_weight {
            return SpliceCandidate::First;
        }
        return SpliceCandidate::Second;
    }
}

/// Structural mutations used in strings and arrays (arrays don't use random value)
pub(crate) enum StructuralMutation {
    ChaoticSelfSplice,
    ChunkDuplication,
    RandomValueDuplication,
    Swap,
}

pub(crate) struct StructuralMutationConfiguration {
    chaotic_self_splice_weight: usize,
    chunk_duplication_weight: usize,
    random_value_duplication_weight: usize,
    #[allow(unused)]
    swap_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting the structural mutation
impl StructuralMutationConfiguration {
    #[allow(unused)]
    pub fn new(
        chaotic_self_splice_weight: usize,
        chunk_duplication_weight: usize,
        random_value_duplication_weight: usize,
        swap_weight: usize,
    ) -> Self {
        let total_weight = chaotic_self_splice_weight
            + chunk_duplication_weight
            + random_value_duplication_weight
            + swap_weight;
        Self {
            chaotic_self_splice_weight,
            chunk_duplication_weight,
            random_value_duplication_weight,
            swap_weight,
            total_weight,
        }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> StructuralMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.chaotic_self_splice_weight {
            return StructuralMutation::ChaoticSelfSplice;
        }
        selector -= self.chaotic_self_splice_weight;
        if selector < self.chunk_duplication_weight {
            return StructuralMutation::ChunkDuplication;
        }
        selector -= self.chunk_duplication_weight;
        if selector < self.random_value_duplication_weight {
            return StructuralMutation::RandomValueDuplication;
        }
        return StructuralMutation::Swap;
    }
}

/// Selection of value or structural mutation. Used in top-level input map and in strings
pub(crate) enum TopLevelMutation {
    Value,
    Structure,
}
pub(crate) struct TopLevelMutationConfiguration {
    value_mutation_weight: usize,
    #[allow(unused)]
    structure_mutation_weight: usize,
    total_weight: usize,
}

/// Configuration for selecting the general mutation mechanism
impl TopLevelMutationConfiguration {
    #[allow(unused)]
    pub fn new(value_mutation_weight: usize, structure_mutation_weight: usize) -> Self {
        let total_weight = value_mutation_weight + structure_mutation_weight;
        Self { value_mutation_weight, structure_mutation_weight, total_weight }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> TopLevelMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.value_mutation_weight {
            return TopLevelMutation::Value;
        }
        return TopLevelMutation::Structure;
    }
}

/// Field-specific configurations

///Field element substitution mutation types
pub(crate) enum FieldElementSubstitutionMutation {
    Zero,
    One,
    MinusOne,
    Dictionary,
    PowerOfTwo,
    PowerOfTwoMinusOne,
}

pub(crate) struct SubstitutionConfiguration {
    substitution_by_zero_weight: usize,
    substitution_by_one_weight: usize,
    substitution_by_minus_one_weight: usize,
    substitution_from_dictionary_weight: usize,
    substitution_by_power_of_2_weight: usize,
    substitution_by_power_of_2_minus_one: usize,
    total_weight: usize,
}

/// Configuration for selecting a substitution mutation
impl SubstitutionConfiguration {
    #[allow(unused)]
    pub fn new(
        substitution_by_zero_weight: usize,
        substitution_by_one_weight: usize,
        substitution_by_minus_one_weight: usize,
        substitution_from_dictionary_weight: usize,
        substitution_by_power_of_2: usize,
        substitution_by_power_of_2_minus_one: usize,
    ) -> Self {
        let total_weight = substitution_by_minus_one_weight
            + substitution_by_one_weight
            + substitution_by_zero_weight
            + substitution_from_dictionary_weight
            + substitution_by_power_of_2
            + substitution_by_power_of_2_minus_one;
        Self {
            substitution_by_zero_weight,
            substitution_by_one_weight,
            substitution_by_minus_one_weight,
            substitution_from_dictionary_weight,
            substitution_by_power_of_2_weight: substitution_by_power_of_2,
            substitution_by_power_of_2_minus_one,
            total_weight,
        }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FieldElementSubstitutionMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.substitution_by_zero_weight {
            return FieldElementSubstitutionMutation::Zero;
        }
        selector -= self.substitution_by_zero_weight;
        if selector < self.substitution_by_one_weight {
            return FieldElementSubstitutionMutation::One;
        }
        selector -= self.substitution_by_one_weight;
        if selector < self.substitution_by_minus_one_weight {
            return FieldElementSubstitutionMutation::MinusOne;
        }
        selector -= self.substitution_by_minus_one_weight;
        if selector < self.substitution_from_dictionary_weight {
            return FieldElementSubstitutionMutation::Dictionary;
        }
        selector -= self.substitution_from_dictionary_weight;
        if selector < self.substitution_by_power_of_2_weight {
            return FieldElementSubstitutionMutation::PowerOfTwo;
        }
        selector -= self.substitution_by_power_of_2_weight;
        debug_assert!(selector < self.substitution_by_power_of_2_minus_one);
        return FieldElementSubstitutionMutation::PowerOfTwoMinusOne;
    }
}

pub(crate) enum FieldElementInversionMutation {
    Additive,
    Multiplicative,
}
pub(crate) struct FieldElementInversionConfiguration {
    additive_inversion_weight: usize,
    #[allow(unused)]
    multiplicative_inversion_weight: usize,
    total_weight: usize,
}

impl FieldElementInversionConfiguration {
    #[allow(unused)]
    pub fn new(additive_inversion_weight: usize, multiplicative_inversion_weight: usize) -> Self {
        let total_weight = additive_inversion_weight + multiplicative_inversion_weight;
        Self { additive_inversion_weight, multiplicative_inversion_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FieldElementInversionMutation {
        let selector = prng.gen_range(0..self.total_weight);
        if selector < self.additive_inversion_weight {
            return FieldElementInversionMutation::Additive;
        }
        return FieldElementInversionMutation::Multiplicative;
    }
}

pub(crate) enum FieldElementPow2Update {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}
pub(crate) struct Pow2UpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    multiplication_weight: usize,
    #[allow(unused)]
    division_weight: usize,
    total_weight: usize,
}

impl Pow2UpdateConfiguration {
    #[allow(unused)]
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
        division_weight: usize,
    ) -> Self {
        let total_weight =
            addition_weight + subtraction_weight + multiplication_weight + division_weight;
        Self {
            addition_weight,
            subtraction_weight,
            multiplication_weight,
            division_weight,
            total_weight,
        }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FieldElementPow2Update {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return FieldElementPow2Update::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return FieldElementPow2Update::Subtraction;
        }
        selector -= self.subtraction_weight;
        if selector < self.multiplication_weight {
            return FieldElementPow2Update::Multiplication;
        }
        return FieldElementPow2Update::Division;
    }
}

pub(crate) enum FieldElementSmallValueUpdate {
    Addition,
    Subtraction,
    Multiplication,
}
pub(crate) struct FieldElementSmallValueUpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    #[allow(unused)]
    multiplication_weight: usize,
    total_weight: usize,
}

impl FieldElementSmallValueUpdateConfiguration {
    #[allow(unused)]
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
    ) -> Self {
        let total_weight = addition_weight + subtraction_weight + multiplication_weight;
        Self { addition_weight, subtraction_weight, multiplication_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FieldElementSmallValueUpdate {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return FieldElementSmallValueUpdate::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return FieldElementSmallValueUpdate::Subtraction;
        }
        return FieldElementSmallValueUpdate::Multiplication;
    }
}

pub(crate) enum FieldElementDictionaryUpdate {
    Addition,
    Subtraction,
    Multiplication,
}
pub(crate) struct FieldElementDictionaryUpdateConfiguration {
    addition_weight: usize,
    subtraction_weight: usize,
    #[allow(unused)]
    multiplication_weight: usize,
    total_weight: usize,
}

impl FieldElementDictionaryUpdateConfiguration {
    #[allow(unused)]
    pub fn new(
        addition_weight: usize,
        subtraction_weight: usize,
        multiplication_weight: usize,
    ) -> Self {
        let total_weight = addition_weight + subtraction_weight;
        Self { addition_weight, subtraction_weight, multiplication_weight, total_weight }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> FieldElementDictionaryUpdate {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.addition_weight {
            return FieldElementDictionaryUpdate::Addition;
        }
        selector -= self.addition_weight;
        if selector < self.subtraction_weight {
            return FieldElementDictionaryUpdate::Subtraction;
        }
        return FieldElementDictionaryUpdate::Multiplication;
    }
}

pub(crate) enum TopLevelFieldElementMutation {
    Substitution,
    Inversion,
    Pow2Update,
    SmallValueUpdate,
    DictionaryUpdate,
}
pub(crate) struct TopLevelFieldElementMutationConfiguration {
    substitution_weight: usize,
    inversion_weight: usize,
    pow_2_update_weight: usize,
    small_value_update_weight: usize,
    #[allow(unused)]
    dictionary_update_weight: usize,
    total_weight: usize,
}

impl TopLevelFieldElementMutationConfiguration {
    #[allow(unused)]
    pub fn new(
        substitution_weight: usize,
        inversion_weight: usize,
        pow_2_update_weight: usize,
        small_value_update_weight: usize,
        dictionary_update_weight: usize,
    ) -> Self {
        let total_weight = substitution_weight
            + inversion_weight
            + pow_2_update_weight
            + small_value_update_weight
            + dictionary_update_weight;
        Self {
            substitution_weight,
            inversion_weight,
            pow_2_update_weight,
            small_value_update_weight,
            dictionary_update_weight,
            total_weight,
        }
    }
    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> TopLevelFieldElementMutation {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.substitution_weight {
            return TopLevelFieldElementMutation::Substitution;
        }
        selector -= self.substitution_weight;
        if selector < self.inversion_weight {
            return TopLevelFieldElementMutation::Inversion;
        }
        selector -= self.inversion_weight;
        if selector < self.pow_2_update_weight {
            return TopLevelFieldElementMutation::Pow2Update;
        }
        selector -= self.pow_2_update_weight;
        if selector < self.small_value_update_weight {
            return TopLevelFieldElementMutation::SmallValueUpdate;
        }
        return TopLevelFieldElementMutation::DictionaryUpdate;
    }
}

pub enum TestCaseSpliceType {
    /// Around 50% for each top-level element
    BalancedTopLevel,
    /// 80/20 for each element at lower level
    UnbalancedFull,
    /// One element merged into the main testcase
    SingleElementImport,
}

pub(crate) struct TestCaseSpliceConfiguration {
    balanced_top_level_weight: usize,
    unbalanced_full_weight: usize,
    #[allow(unused)]
    single_element_import_weight: usize,
    total_weight: usize,
}

impl TestCaseSpliceConfiguration {
    #[allow(unused)]
    pub fn new(
        balanced_top_level_weight: usize,
        unbalanced_full_weight: usize,
        single_element_import_weight: usize,
    ) -> Self {
        let total_weight =
            balanced_top_level_weight + unbalanced_full_weight + single_element_import_weight;
        Self {
            balanced_top_level_weight,
            unbalanced_full_weight,
            single_element_import_weight,
            total_weight,
        }
    }

    /// Select a mutation according to weights
    pub fn select(&self, prng: &mut XorShiftRng) -> TestCaseSpliceType {
        let mut selector = prng.gen_range(0..self.total_weight);
        if selector < self.balanced_top_level_weight {
            return TestCaseSpliceType::BalancedTopLevel;
        }
        selector -= self.balanced_top_level_weight;
        if selector < self.unbalanced_full_weight {
            return TestCaseSpliceType::UnbalancedFull;
        }
        return TestCaseSpliceType::SingleElementImport;
    }
}

/// Default configurations for all mutations that are currently used

pub(crate) const BASIC_SPLICE_MUTATION_CONFIGURATION: SpliceMutationConfiguration =
    SpliceMutationConfiguration {
        position_preserving_mutation_weight: 1,
        random_chunks_weight: 1,
        total_weight: 1 + 1,
    };
pub(crate) const BASIC_UNBALANCED_ARRAY_SPLICE_MUTATION_CONFIGURATION:
    UnbalancedArraySpliceConfiguration = UnbalancedArraySpliceConfiguration {
    array_specific_weight: 11,
    recurse_weight: 9,
    total_weight: 11 + 9,
};
pub(crate) const BASIC_BYTE_VALUE_MUTATION_CONFIGURATION: ByteValueMutationConfiguration =
    ByteValueMutationConfiguration {
        random_byte_mutation_weight: 1,
        dictionary_byte_mutation_weight: 3,
        total_weight: 1 + 3,
    };

pub(crate) const DICTIONARY_EMPTY_BYTE_VALUE_MUTATION_CONFIGURATION:
    ByteValueMutationConfiguration = ByteValueMutationConfiguration {
    random_byte_mutation_weight: 1,
    dictionary_byte_mutation_weight: 0,
    total_weight: 1,
};

pub(crate) const BASIC_SPLICE_CANDIDATE_PRIORITIZATION_CONFIGURATION:
    SpliceCandidatePrioritizationConfiguration = SpliceCandidatePrioritizationConfiguration {
    first_weight: 11,
    second_weight: 10,
    total_weight: 11 + 10,
};

pub(crate) const BASIC_STRUCTURE_MUTATION_CONFIGURATION: StructuralMutationConfiguration =
    StructuralMutationConfiguration {
        chaotic_self_splice_weight: 3,
        chunk_duplication_weight: 2,
        random_value_duplication_weight: 1,
        swap_weight: 3,
        total_weight: 3 + 2 + 1 + 3,
    };

pub(crate) const BASIC_TOP_LEVEL_MUTATION_CONFIGURATION: TopLevelMutationConfiguration =
    TopLevelMutationConfiguration {
        value_mutation_weight: 7,
        structure_mutation_weight: 3,
        total_weight: 7 + 3,
    };

/// Field-specific mutation configurations

pub(crate) const BASIC_FIELD_SUBSTITUTION_CONFIGURATION: SubstitutionConfiguration =
    SubstitutionConfiguration {
        substitution_by_zero_weight: 20,
        substitution_by_one_weight: 20,
        substitution_by_minus_one_weight: 20,
        substitution_from_dictionary_weight: 20,
        substitution_by_power_of_2_weight: 254,
        substitution_by_power_of_2_minus_one: 254,
        total_weight: 20 + 20 + 20 + 20 + 254 + 254,
    };
pub(crate) const BASIC_FIELD_INVERSION_CONFIGURATION: FieldElementInversionConfiguration =
    FieldElementInversionConfiguration {
        additive_inversion_weight: 10,
        multiplicative_inversion_weight: 1,
        total_weight: 10 + 1,
    };

pub(crate) const BASIC_FIELD_ELEMENT_POW_2_UPDATE_CONFIGURATION: Pow2UpdateConfiguration =
    Pow2UpdateConfiguration {
        addition_weight: 3,
        subtraction_weight: 3,
        multiplication_weight: 2,
        division_weight: 1,
        total_weight: 3 + 3 + 2 + 1,
    };
pub(crate) const BASIC_FIELD_ELEMENT_SMALL_VALUE_UPDATE_CONFIGURATION:
    FieldElementSmallValueUpdateConfiguration = FieldElementSmallValueUpdateConfiguration {
    addition_weight: 3,
    subtraction_weight: 3,
    multiplication_weight: 1,
    total_weight: 3 + 3 + 1,
};

pub(crate) const BASIC_FIELD_ELEMENT_DICTIONARY_UPDATE_CONFIGURATION:
    FieldElementDictionaryUpdateConfiguration = FieldElementDictionaryUpdateConfiguration {
    addition_weight: 2,
    subtraction_weight: 2,
    multiplication_weight: 1,
    total_weight: 2 + 2 + 1,
};

pub(crate) const BASIC_TOPLEVEL_FIELD_ELEMENT_MUTATION_CONFIGURATION:
    TopLevelFieldElementMutationConfiguration = TopLevelFieldElementMutationConfiguration {
    substitution_weight: 10,
    inversion_weight: 1,
    pow_2_update_weight: 5,
    small_value_update_weight: 10,
    dictionary_update_weight: 10,
    total_weight: 10 + 1 + 5 + 10 + 10,
};

pub(crate) const BASIC_TESTCASE_SPLICE_CONFIGURATION: TestCaseSpliceConfiguration =
    TestCaseSpliceConfiguration {
        balanced_top_level_weight: 1,
        unbalanced_full_weight: 1,
        single_element_import_weight: 2,
        total_weight: 1 + 1 + 2,
    };

/// Generic vector structural mutation configuration (random value duplication weight MUST stay zero)
pub(crate) const BASIC_VECTOR_STRUCTURE_MUTATION_CONFIGURATION: StructuralMutationConfiguration =
    StructuralMutationConfiguration {
        chaotic_self_splice_weight: 3,
        chunk_duplication_weight: 2,
        random_value_duplication_weight: 0,
        swap_weight: 3,
        total_weight: 3 + 2 + 0 + 3,
    };
