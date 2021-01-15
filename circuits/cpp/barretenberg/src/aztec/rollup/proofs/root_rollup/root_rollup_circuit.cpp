#include "../../constants.hpp"
#include "../rollup/rollup_circuit.hpp"
#include "./root_rollup_circuit.hpp"
#include "../inner_proof_data.hpp"
#include <stdlib/merkle_tree/membership.hpp>
#include <common/throw_or_abort.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;

void check_root_tree_updated(Composer& composer,
                             merkle_tree::hash_path const& new_data_roots_path,
                             merkle_tree::hash_path const& old_data_roots_path,
                             field_ct const& rollup_id,
                             field_ct const& new_data_root,
                             field_ct const& new_data_roots_root,
                             field_ct const& old_data_roots_root)
{
    auto empty_tree_value = byte_array_ct(&composer, 64);
    auto new_data_root_arr = byte_array_ct(new_data_root);
    auto one = field_ct(witness_ct(&composer, 1));
    auto index = byte_array_ct(rollup_id + one);
    update_membership(composer,
                      new_data_roots_root,
                      new_data_roots_path,
                      new_data_root_arr,
                      old_data_roots_root,
                      old_data_roots_path,
                      empty_tree_value,
                      index,
                      __FUNCTION__);
}

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& root_rollup,
                                            size_t inner_rollup_size,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    recursion_output<bn254> recursion_output;

    std::vector<field_ct> inner_proof_public_inputs;
    uint32_ct num_inner_proofs = witness_ct(&composer, root_rollup.num_inner_proofs);
    field_ct rollup_id = witness_ct(&composer, root_rollup.rollup_id);
    field_ct rollup_size = witness_ct(&composer, inner_rollup_size * root_rollup.rollups.size());
    field_ct data_start_index = witness_ct(&composer, 0);
    field_ct old_data_root = witness_ct(&composer, 0);
    field_ct new_data_root = witness_ct(&composer, 0);
    field_ct old_null_root = witness_ct(&composer, 0);
    field_ct new_null_root = witness_ct(&composer, 0);
    field_ct old_root_root = witness_ct(&composer, root_rollup.old_data_roots_root);
    field_ct new_root_root = witness_ct(&composer, root_rollup.new_data_roots_root);
    field_ct num_txs = field_ct::from_witness_index(&composer, composer.zero_idx);

    auto total_tx_fees = std::vector<field_ct>(NUM_ASSETS, field_ct::from_witness_index(&composer, composer.zero_idx));
    auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);

    for (size_t i = 0; i < root_rollup.rollups.size(); ++i) {
        auto recursive_verification_key =
            plonk::stdlib::recursion::verification_key<bn254>::from_constants(&composer, inner_verification_key);
        recursion_output =
            verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(&composer,
                                                                          recursive_verification_key,
                                                                          recursive_manifest,
                                                                          waffle::plonk_proof{ root_rollup.rollups[i] },
                                                                          recursion_output);

        auto public_inputs = recursion_output.public_inputs;
        auto tx_index = uint32_ct(static_cast<uint32_t>(i));
        auto is_real = num_inner_proofs > tx_index;
        auto data_start_index_inner = public_inputs[2];
        auto old_data_root_inner = public_inputs[3];
        auto new_data_root_inner = public_inputs[4];
        auto old_null_root_inner = public_inputs[5];
        auto new_null_root_inner = public_inputs[6];
        auto old_root_root_inner = public_inputs[7];
        for (size_t j = 0; j < NUM_ASSETS; ++j) {
            total_tx_fees[j] += public_inputs[9 + j];
        }
        auto num_txs_inner = public_inputs[9 + NUM_ASSETS];

        for (size_t j = 0; j < InnerProofFields::NUM_PUBLISHED * inner_rollup_size; ++j) {
            inner_proof_public_inputs.push_back(public_inputs[10 + NUM_ASSETS + j]);
        }

        // Every real inner proof should use the root tree root we've input.
        auto valid_root_root = (!is_real || old_root_root_inner == old_root_root).normalize();
        composer.assert_equal_constant(valid_root_root.witness_index, 1, format("inconsistent_root_roots_", i));

        // Check inner proof has valid number of txs.
        auto is_leading = tx_index < num_inner_proofs;
        auto is_last = tx_index == num_inner_proofs - 1;
        auto is_padding = tx_index >= num_inner_proofs;

        auto valid_leading = is_leading && (num_txs_inner == inner_rollup_size);
        auto valid_last = is_last && !num_txs_inner.is_zero();
        auto valid_padding = is_padding && num_txs_inner.is_zero();
        auto valid_num_txs = valid_leading || valid_last || valid_padding;

        composer.assert_equal_constant(valid_num_txs.normalize().witness_index, 1, format("invalid_num_txs_", i));

        if (i == 0) {
            // The first proof should always be real.
            composer.assert_equal_constant(is_real.witness_index, 1);
            data_start_index = data_start_index_inner;
            old_data_root = old_data_root_inner;
            new_data_root = new_data_root_inner;
            old_null_root = old_null_root_inner;
            new_null_root = new_null_root_inner;
            num_txs = num_txs_inner;
        } else {
            auto valid_data_start_index =
                is_padding || data_start_index_inner == (data_start_index + (i * inner_rollup_size * 2));
            auto valid_old_data_root = is_padding || old_data_root_inner == new_data_root;
            auto valid_old_null_root = is_padding || old_null_root_inner == new_null_root;

            composer.assert_equal_constant(
                valid_data_start_index.normalize().witness_index, 1, format("incorrect_data_start_index_", i));
            composer.assert_equal_constant(
                valid_old_data_root.normalize().witness_index, 1, format("inconsistent_data_roots_", i));
            composer.assert_equal_constant(
                valid_old_null_root.normalize().witness_index, 1, format("inconsistent_null_roots_", i));

            new_data_root = (new_data_root_inner * is_real) + (new_data_root * is_padding);
            new_null_root = (new_null_root_inner * is_real) + (new_null_root * is_padding);
            num_txs += num_txs_inner;
            composer.create_range_constraint(num_txs.get_witness_index(), MAX_TXS_BIT_LENGTH);
        }
    }

    auto new_data_roots_path = create_witness_hash_path(composer, root_rollup.new_data_roots_path);
    auto old_data_roots_path = create_witness_hash_path(composer, root_rollup.old_data_roots_path);
    check_root_tree_updated(
        composer, new_data_roots_path, old_data_roots_path, rollup_id, new_data_root, new_root_root, old_root_root);

    composer.set_public_input(rollup_id.witness_index);
    composer.set_public_input(rollup_size.witness_index);
    composer.set_public_input(data_start_index.witness_index);
    composer.set_public_input(old_data_root.witness_index);
    composer.set_public_input(new_data_root.witness_index);
    composer.set_public_input(old_null_root.witness_index);
    composer.set_public_input(new_null_root.witness_index);
    composer.set_public_input(old_root_root.witness_index);
    composer.set_public_input(new_root_root.witness_index);
    for (auto total_tx_fee : total_tx_fees) {
        composer.set_public_input(total_tx_fee.witness_index);
    }
    composer.set_public_input(num_txs.get_witness_index());

    for (auto& inp : inner_proof_public_inputs) {
        composer.set_public_input(inp.witness_index);
    }
    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
