#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"
#include "barretenberg/proof_system/relations/relation_types.hpp"

namespace proof_system::honk::sumcheck {

template <typename FF_> class ECCVMSetRelationBase {
  public:
    using FF = FF_;
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 19;

    static constexpr size_t LEN_1 = RELATION_LENGTH; // grand product construction sub-relation
    static constexpr size_t LEN_2 = RELATION_LENGTH; // left-shiftable polynomial sub-relation
    template <template <size_t...> typename AccumulatorTypesContainer>
    using GetAccumulatorTypes = AccumulatorTypesContainer<LEN_1, LEN_1>;
    template <typename T> using Accumulator = typename std::tuple_element<0, typename T::Accumulators>::type;

    template <typename AccumulatorTypes>
    static Accumulator<AccumulatorTypes> convert_to_wnaf(const auto& s0, const auto& s1)
    {
        auto t = s0 + s0;
        t += t;
        t += s1;

        auto naf = t + t - 15;
        return naf;
    }

    inline static auto& get_grand_product_polynomial(auto& input) { return input.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& input) { return input.z_perm_shift; }

    template <typename AccumulatorTypes>
    static Accumulator<AccumulatorTypes> compute_permutation_numerator(const auto& extended_edges,
                                                                       const RelationParameters<FF>& relation_params,
                                                                       size_t index = 0);

    template <typename AccumulatorTypes>
    static Accumulator<AccumulatorTypes> compute_permutation_denominator(const auto& extended_edges,
                                                                         const RelationParameters<FF>& relation_params,
                                                                         size_t index = 0);

    template <typename AccumulatorTypes>
    static void accumulate(typename AccumulatorTypes::Accumulators& accumulator,
                           const auto& extended_edges,
                           const RelationParameters<FF>& relation_params,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMSetRelation = Relation<ECCVMSetRelationBase<FF>>;

} // namespace proof_system::honk::sumcheck
