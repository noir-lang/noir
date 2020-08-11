#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct join_split_tx {
    uint32_t public_input;
    uint32_t public_output;
    uint32_t num_input_notes;
    std::array<uint32_t, 2> input_index;
    barretenberg::fr merkle_root;
    std::array<merkle_tree::fr_hash_path, 2> input_path;
    std::array<tx_note, 2> input_note;
    std::array<tx_note, 2> output_note;

    uint32_t account_index;
    grumpkin::g1::affine_element signing_pub_key;
    merkle_tree::fr_hash_path account_path;
    crypto::schnorr::signature signature;

    barretenberg::fr input_owner;
    barretenberg::fr output_owner;
};

void read(uint8_t const*& it, join_split_tx& tx);
void write(std::vector<uint8_t>& buf, join_split_tx const& tx);

bool operator==(join_split_tx const& lhs, join_split_tx const& rhs);
std::ostream& operator<<(std::ostream& os, join_split_tx const& tx);

} // namespace join_split
} // namespace client_proofs
} // namespace rollup

// Optimisation of to_buffer that reserves full amount now for optimal efficiency.
inline std::vector<uint8_t> to_buffer(rollup::client_proofs::join_split::join_split_tx const& tx)
{
    std::vector<uint8_t> buf;
    buf.reserve(64 + (4 * 5) + 32 + (64 * 32 * 2) + (100 * 4) + 64 + 64);
    write(buf, tx);
    return buf;
}
