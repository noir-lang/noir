#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct join_split_data {
    uint32_t public_input;
    uint32_t public_output;
    barretenberg::fr merkle_root;
    std::array<uint8_t, 64> new_note1;
    std::array<uint8_t, 64> new_note2;
    uint128_t nullifier1;
    uint128_t nullifier2;
    barretenberg::fr input_owner;
    barretenberg::fr output_owner;

    join_split_data(std::vector<uint8_t> const& proof_data);
};

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
