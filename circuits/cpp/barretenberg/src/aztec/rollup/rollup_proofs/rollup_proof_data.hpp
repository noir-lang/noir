#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct inner_proof_data {
    uint32_t public_input;
    uint32_t public_output;
    std::array<uint8_t, 64> new_note1;
    std::array<uint8_t, 64> new_note2;
    uint128_t nullifier1;
    uint128_t nullifier2;
    barretenberg::fr input_owner;
    barretenberg::fr output_owner;
};

struct rollup_proof_data {
    uint32_t rollup_id;
    uint32_t data_start_index;
    fr old_data_root;
    fr new_data_root;
    fr old_null_root;
    fr new_null_root;
    fr old_data_roots_root;
    fr new_data_roots_root;
    uint32_t num_txs;
    std::vector<inner_proof_data> inner_proofs;

    rollup_proof_data(std::vector<uint8_t> const& proof_data);
};

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
