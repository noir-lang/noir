#pragma once

#include <cstdint>

#include "../../transcript/transcript_wrappers.hpp"
#include "../../plonk/proof_system/types/prover_settings.hpp"
#include <proof_system/flavor/flavor.hpp>

namespace honk {

class standard_verifier_settings : public waffle::standard_settings {
  public:
    typedef barretenberg::fr fr;
    typedef transcript::StandardTranscript Transcript;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;

    static constexpr size_t num_polys = bonk::StandardArithmetization::NUM_POLYNOMIALS;
};

} // namespace honk