#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class ECCVMSetRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        21, // grand product construction sub-relation
        21  // left-shiftable polynomial sub-relation
    };
    /**
     * @brief For ZK-Flavors: Upper bound on the degrees of subrelations considered as polynomials only in witness
polynomials,
     * i.e. all selectors and public polynomials are treated as constants. The subrelation witness degree does not
     * exceed the subrelation partial degree given by SUBRELATION_PARTIAL_LENGTH - 1.
     */
    static constexpr std::array<size_t, 2> SUBRELATION_WITNESS_DEGREES{
        20, // grand product construction sub-relation
        20  // left-shiftable polynomial sub-relation
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

    template <typename Accumulator, typename AllEntities, typename Parameters>
    static Accumulator compute_grand_product_numerator(const AllEntities& in, const Parameters& params);

    template <typename Accumulator, typename AllEntities, typename Parameters>
    static Accumulator compute_grand_product_denominator(const AllEntities& in, const Parameters& params);

    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMSetRelation = Relation<ECCVMSetRelationImpl<FF>>;

} // namespace bb