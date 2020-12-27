#pragma once
#include "rollup_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

field_ct check_nullifiers_inserted(Composer& composer,
                                   std::vector<fr> const& new_null_roots,
                                   std::vector<fr_hash_path> const& old_null_paths,
                                   std::vector<fr_hash_path> const& new_null_paths,
                                   uint32_ct const& num_txs,
                                   field_ct latest_null_root,
                                   std::vector<field_ct> const& new_null_indicies);

void check_data_tree_updated(Composer& composer,
                             size_t rollup_size,
                             merkle_tree::hash_path const& new_data_path,
                             merkle_tree::hash_path const& old_data_path,
                             std::vector<byte_array_ct> const& new_data_values,
                             field_ct const& old_data_root,
                             field_ct const& new_data_root,
                             field_ct const& data_start_index);

recursion_output<bn254> rollup_circuit(Composer& composer,
                                       rollup_tx const& proofs,
                                       std::vector<std::shared_ptr<waffle::verification_key>> const& verification_keys,
                                       size_t rollup_size);

} // namespace rollup
} // namespace proofs
} // namespace rollup
