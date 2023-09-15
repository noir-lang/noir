#pragma once
#include "barretenberg/proof_system/relations/relation_types.hpp"

namespace proof_system::honk::sumcheck {
/**
 * @brief ECCVMWnafRelationBase evaluates relations that convert scalar multipliers into 4-bit WNAF slices
 * @details Each WNAF slice is a 4-bit slice representing one of 16 integers { -15, -13, ..., 15 }
 * Each WNAF slice is represented via two 2-bit columns (precompute_s1hi, ..., precompute_s4lo)
 * One 128-bit scalar multiplier is processed across 8 rows, indexed by a round variable.
 * The following table describes the structure for one scalar.
 *
 * | point_transition | round | slices          | skew   | scalar_sum                      |
 * | ---------------- | ----- | --------------- | ------ | ------------------------------- |
 * | 0                | 0     | s0,s1,s2,s3     | 0      | 0                               |
 * | 0                | 1     | s4,s5,s6,s7     | 0      | \sum_{i=0}^4 16^i * s_{31 - i}  |
 * | 0                | 2     | s8,s9,s10,s11   | 0      | \sum_{i=0}^8 16^i * s_{31 - i}  |
 * | 0                | 3     | s12,s13,s14,s14 | 0      | \sum_{i=0}^12 16^i * s_{31 - i} |
 * | 0                | 4     | s16,s17,s18,s19 | 0      | \sum_{i=0}^16 16^i * s_{31 - i} |
 * | 0                | 5     | s20,s21,s22,s23 | 0      | \sum_{i=0}^20 16^i * s_{31 - i} |
 * | 0                | 6     | s24,s25,s26,s27 | 0      | \sum_{i=0}^24 16^i * s_{31 - i} |
 * | 1                | 7     | s28,s29,s30,s31 | s_skew | \sum_{i=0}^28 16^i * s_{31 - i} |
 *
 * The value of the input scalar is equal to the following:
 *
 * scalar = 2^16 * scalar_sum + 2^12 * s31 + 2^8 * s30 + 2^4 * s29 + s28 - s_skew
 * We use a set equality check in `ecc_set_relation.hpp` to validate the above value maps to the correct input
 * scalar for a given value of `pc`.
 *
 * The column `point_transition` is committed to by the Prover, we must constrain it is correctly computed (see
 * `ecc_point_table_relation.cpp` for details)
 *
 * @tparam FF
 */
template <typename FF_> class ECCVMWnafRelationBase {
  public:
    using FF = FF_;
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5;

    static constexpr size_t LEN_1 = 5;
    template <template <size_t...> typename AccumulatorTypesContainer>
    using GetAccumulatorTypes = AccumulatorTypesContainer<LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1,
                                                          LEN_1>;

    template <typename AccumulatorTypes>
    static void accumulate(typename AccumulatorTypes::Accumulators& accumulator,
                           const auto& extended_edges,
                           const RelationParameters<FF>& /*unused*/,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMWnafRelation = Relation<ECCVMWnafRelationBase<FF>>;

} // namespace proof_system::honk::sumcheck
