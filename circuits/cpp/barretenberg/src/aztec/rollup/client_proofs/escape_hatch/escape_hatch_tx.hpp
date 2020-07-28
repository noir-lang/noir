#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace escape_hatch {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

struct escape_hatch_tx {
    uint256_t public_output;
    uint32_t num_input_notes;
    std::array<uint32_t, 2> input_index;
    barretenberg::fr old_data_root;
    std::array<merkle_tree::fr_hash_path, 2> input_path;
    std::array<tx_note, 2> input_note;
    crypto::schnorr::signature signature;
    barretenberg::fr public_owner;

    uint32_t account_index;
    merkle_tree::fr_hash_path account_path;
    merkle_tree::fr_hash_path account_nullifier_path;

    grumpkin::g1::affine_element signing_pub_key;

    barretenberg::fr old_nullifier_merkle_root;
    std::array<barretenberg::fr, 2> new_null_roots; // final root is the new_null_root
    std::array<merkle_tree::fr_hash_path, 2> current_nullifier_paths;
    std::array<merkle_tree::fr_hash_path, 2> new_nullifier_paths;

    fr new_data_root;
    fr old_data_roots_root;
    fr new_data_roots_root;
};

void read(uint8_t const*& it, escape_hatch_tx& tx);
void write(std::vector<uint8_t>& buf, escape_hatch_tx const& tx);

bool operator==(escape_hatch_tx const& lhs, escape_hatch_tx const& rhs);
std::ostream& operator<<(std::ostream& os, escape_hatch_tx const& tx);

} // namespace escape_hatch
} // namespace client_proofs
} // namespace rollup

inline std::vector<uint8_t> to_buffer(rollup::client_proofs::escape_hatch::escape_hatch_tx const& tx)
{
    std::vector<uint8_t> buf;
    write(buf, tx);
    return buf;
}
