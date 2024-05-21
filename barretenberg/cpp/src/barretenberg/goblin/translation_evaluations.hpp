#pragma once
#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/fields/field_conversion.hpp"

namespace bb {
/**
 * @brief Stores the evaluations from ECCVM, checked against the translator evaluations as a final step of translator.
 *
 * @tparam BF The base field of the curve, translation evaluations are represented in the base field.
 * @tparam FF The scalar field of the curve, used in Goblin to help convert the proof into a buffer for ACIR.
 */
template <typename BF, typename FF> struct TranslationEvaluations_ {
    BF op, Px, Py, z1, z2;
    static constexpr uint32_t NUM_EVALUATIONS = 5;
    static size_t size() { return field_conversion::calc_num_bn254_frs<BF>() * NUM_EVALUATIONS; }
    std::vector<FF> to_buffer() const
    {
        std::vector<FF> result;
        result.reserve(size());
        const auto insert = [&result](const BF& elt) {
            std::vector<FF> buf = field_conversion::convert_to_bn254_frs(elt);
            result.insert(result.end(), buf.begin(), buf.end());
        };
        insert(op);
        insert(Px);
        insert(Py);
        insert(z1);
        insert(z2);
        return result;
    }
};
} // namespace bb