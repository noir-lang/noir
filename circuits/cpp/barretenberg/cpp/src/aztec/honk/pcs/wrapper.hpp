#pragma once

#include "commitment_key.hpp"
#include "gemini/gemini.hpp"
#include <ecc/curves/bn254/g1.hpp>

namespace honk {

struct OpeningProof {
    std::vector<barretenberg::g1::affine_element> gemini;
    barretenberg::g1::affine_element shplonk;
    barretenberg::g1::affine_element kzg;
};

} // namespace honk