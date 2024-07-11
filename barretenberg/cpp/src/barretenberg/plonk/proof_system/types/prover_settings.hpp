#pragma once
#include "barretenberg/plonk_honk_shared/arithmetization/standard_arithmetization.hpp"
#include "barretenberg/transcript/transcript.hpp"
namespace bb::plonk {
class settings_base {
  public:
    static constexpr bool requires_shifted_wire(const uint64_t wire_shift_settings, const uint64_t wire_index)
    {
        return (((wire_shift_settings >> (wire_index)) & 1UL) == 1UL);
    }
};

class standard_settings : public settings_base {
  public:
    using Arithmetization = StandardArith<bb::fr>;
    using Flavor = plonk::flavor::Standard;
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
    static constexpr size_t program_width = 3;
    static constexpr size_t num_shifted_wire_evaluations = 1;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr size_t num_roots_cut_out_of_vanishing_polynomial = 4;
    static constexpr bool is_plookup = false;
};

class ultra_settings : public settings_base {
  public:
    using Flavor = plonk::flavor::Ultra;
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
    static constexpr size_t program_width = 4;
    static constexpr size_t num_shifted_wire_evaluations = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr size_t num_roots_cut_out_of_vanishing_polynomial = 4;
    static constexpr bool is_plookup = true;
};

// Only needed because ultra-to-standard recursion requires us to use a Pedersen hash which is common to both Ultra &
// Standard plonk i.e. the non-ultra version.
class ultra_to_standard_settings : public ultra_settings {
  public:
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
};

// Only needed because ultra-to-standard recursion requires us to use a Pedersen hash which is common to both Ultra &
// Standard plonk i.e. the non-ultra version.
class ultra_with_keccak_settings : public ultra_settings {
  public:
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
};

} // namespace bb::plonk
