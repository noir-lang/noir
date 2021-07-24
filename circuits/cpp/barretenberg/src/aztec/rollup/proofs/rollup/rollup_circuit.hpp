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
                                   std::vector<field_ct> const& new_null_roots,
                                   std::vector<merkle_tree::hash_path> const& old_null_paths,
                                   uint32_ct const& num_txs,
                                   field_ct latest_null_root,
                                   std::vector<field_ct> const& new_null_indicies);

recursion_output<bn254> rollup_circuit(Composer& composer,
                                       rollup_tx const& proofs,
                                       std::vector<std::shared_ptr<waffle::verification_key>> const& verification_keys,
                                       size_t rollup_size);

void add_tx_padding_public_inputs(Composer& composer);

} // namespace rollup
} // namespace proofs
} // namespace rollup
