#pragma once

#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "gemini/gemini.hpp"

namespace proof_system::honk {

struct OpeningProof {
    std::vector<bb::g1::affine_element> gemini;
    bb::g1::affine_element shplonk;
    bb::g1::affine_element kzg;
};

} // namespace proof_system::honk
