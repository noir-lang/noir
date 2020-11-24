#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>
#include "../join_split/join_split_tx.hpp"

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

struct escape_hatch_tx {
    join_split::join_split_tx js_tx;

    uint32_t rollup_id;
    uint32_t data_start_index;
    fr new_data_root;
    fr_hash_path old_data_path;
    fr_hash_path new_data_path;

    fr old_null_root;
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;
    std::vector<fr_hash_path> new_null_paths;

    fr old_data_roots_root;
    fr new_data_roots_root;
    fr_hash_path old_data_roots_path;
    fr_hash_path new_data_roots_path;

    // bool operator==(escape_hatch_tx const& rhs) const = default;
};

void read(uint8_t const*& it, escape_hatch_tx& tx);
void write(std::vector<uint8_t>& buf, escape_hatch_tx const& tx);

bool operator==(escape_hatch_tx const& lhs, escape_hatch_tx const& rhs);
std::ostream& operator<<(std::ostream& os, escape_hatch_tx const& tx);

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
