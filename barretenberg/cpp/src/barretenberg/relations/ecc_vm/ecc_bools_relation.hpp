#pragma once

#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

/**
 * @brief ECCVMBoolsRelationImpl evaluates the correctness of ECCVM boolean checks
 *
 * @details There are a lot of columns in ECCVM that are boolean. As these are all low-degree we place them in a
 * separate relation class
 * @tparam FF
 */
template <typename FF_> class ECCVMBoolsRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 19> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    };

    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& /* unused */,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMBoolsRelation = Relation<ECCVMBoolsRelationImpl<FF>>;

} // namespace bb
