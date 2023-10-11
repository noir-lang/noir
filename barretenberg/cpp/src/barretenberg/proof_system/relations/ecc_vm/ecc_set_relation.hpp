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

    static constexpr std::array<size_t, 2> SUBRELATION_LENGTHS{
        19, // grand product construction sub-relation
        19  // left-shiftable polynomial sub-relation
    };

    template <typename Accumulator> static Accumulator convert_to_wnaf(const auto& s0, const auto& s1)
    {
        auto t = s0 + s0;
        t += t;
        t += s1;

        auto naf = t + t - 15;
        return naf;
    }

    inline static auto& get_grand_product_polynomial(auto& input) { return input.z_perm; }
    inline static auto& get_shifted_grand_product_polynomial(auto& input) { return input.z_perm_shift; }

    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_permutation_numerator(const AllEntities& in,
                                                     const RelationParameters<FF>& relation_params);

    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_permutation_denominator(const AllEntities& in,
                                                       const RelationParameters<FF>& relation_params);

    template <typename ContainerOverSubrelations, typename AllEntities>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const RelationParameters<FF>& relation_params,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMSetRelation = Relation<ECCVMSetRelationBase<FF>>;

} // namespace proof_system::honk::sumcheck
