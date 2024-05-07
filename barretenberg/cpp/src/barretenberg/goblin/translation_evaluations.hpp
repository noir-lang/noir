#pragma once
#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/fields/field_conversion.hpp"

namespace bb {
struct TranslationEvaluations {
    fq op, Px, Py, z1, z2;
    static constexpr uint32_t NUM_EVALUATIONS = 5;
    static size_t size() { return field_conversion::calc_num_bn254_frs<fq>() * NUM_EVALUATIONS; }
    std::vector<fr> to_buffer() const
    {
        std::vector<fr> result;
        result.reserve(size());
        const auto insert = [&result](const fq& elt) {
            std::vector<fr> buf = field_conversion::convert_to_bn254_frs(elt);
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