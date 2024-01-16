#pragma once
#include "barretenberg/ecc/curves/bn254/fq.hpp"

namespace bb {
struct TranslationEvaluations {
    fq op, Px, Py, z1, z2;
    std::vector<uint8_t> to_buffer()
    {
        std::vector<uint8_t> result(5 * sizeof(fq));
        const auto insert = [&result](const fq& elt) {
            std::vector<uint8_t> buf = elt.to_buffer();
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