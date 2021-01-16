#pragma once
#include "./root_rollup_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

void check_root_tree_updated(Composer& composer,
                             merkle_tree::hash_path const& new_data_roots_path,
                             merkle_tree::hash_path const& old_data_roots_path,
                             field_ct const& rollup_id,
                             field_ct const& new_data_root,
                             field_ct const& new_data_roots_root,
                             field_ct const& old_data_roots_root);

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& rollups,
                                            size_t inner_rollup_size,
                                            size_t outer_rollup_size,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
