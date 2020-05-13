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
    std::vector<uint8_t> new_note1;
    std::vector<uint8_t> new_note2;
    uint128_t nullifier1;
    uint128_t nullifier2;

    join_split_data(std::vector<uint8_t> const& proof_data);
};

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
