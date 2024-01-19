#pragma once

#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief ECCVMTranscriptRelationImpl evaluates the correctness of the ECCVM transcript columns
 *
 * @details The transcript relations directly evaluate the correctness of `add, eq, reset` operations.
 * `mul` operations are lazily evaluated. The output of multiscalar multiplications is present in
 * `transcript_msm_x, transcript_msm_y` columns. A set equality check is used to validate these
 * have been correctly read from a table produced by the relations in `ecc_msm_relation.hpp`.
 *
 * Sequential `mul` opcodes are interpreted as a multiscalar multiplication.
 * The column `transcript_msm_count` tracks the number of muls in a given multiscalar multiplication.
 *
 * The column `transcript_pc` tracks a "point counter" value, that describes the number of multiplications
 * that must be evaluated.
 *
 * One mul opcode can generate up to TWO multiplications. Each 128-bit scalar `z1, z2` is treated as an independent
 * mul. The purpose of this is to reduce the length of the MSM algorithm evalauted in `ecc_msm_relation.hpp` to 128
 * bits (from 256 bits). Many scalar muls required to recursively verify a proof are only 128-bits in length; this
 * prevents us doing redundant computation.
 * @tparam FF
 */
template <typename FF_> class ECCVMTranscriptRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 35> SUBRELATION_PARTIAL_LENGTHS{
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    };

    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& /* unused */,
                           const FF& scaling_factor);

    // TODO(@zac-williamson #2609 find more generic way of doing this)
    static constexpr FF get_curve_b()
    {
        if constexpr (FF::modulus == bb::fq::modulus) {
            return bb::g1::curve_b;
        } else if constexpr (FF::modulus == grumpkin::fq::modulus) {
            return grumpkin::g1::curve_b;
        } else {
            return 0;
        }
    }
};

template <typename FF> using ECCVMTranscriptRelation = Relation<ECCVMTranscriptRelationImpl<FF>>;

} // namespace bb::honk::sumcheck
