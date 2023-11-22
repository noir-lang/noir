#pragma once
#include "barretenberg/ecc/curves/bn254/fq.hpp"

namespace barretenberg {
struct TranslationEvaluations {
    fq op, Px, Py, z1, z2;
};
} // namespace barretenberg