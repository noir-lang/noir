#include "rollup_circuit.hpp"
#include <stdlib/merkle_tree/membership.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;

void propagate_inner_proof_public_inputs(Composer& composer, std::vector<field_ct> const& public_inputs)
{
    composer.set_public_input(public_inputs[0].witness_index);
    composer.set_public_input(public_inputs[1].witness_index);
    composer.set_public_input(public_inputs[2].witness_index);
    composer.set_public_input(public_inputs[3].witness_index);
    composer.set_public_input(public_inputs[4].witness_index);
    composer.set_public_input(public_inputs[5].witness_index);
    composer.set_public_input(public_inputs[7].witness_index);
    composer.set_public_input(public_inputs[8].witness_index);
}

std::vector<recursion_output<bn254>> rollup_circuit(
    Composer& composer,
    rollup_tx const& rollup,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    size_t rollup_size,
    bool can_throw)
{
    auto rollup_id = field_ct(public_witness_ct(&composer, rollup.rollup_id));
    auto data_start_index = field_ct(public_witness_ct(&composer, rollup.data_start_index));
    auto old_data_root = field_ct(public_witness_ct(&composer, rollup.old_data_root));
    auto new_data_root = field_ct(public_witness_ct(&composer, rollup.new_data_root));
    auto old_null_root = field_ct(public_witness_ct(&composer, rollup.old_null_root));
    public_witness_ct(&composer, rollup.new_null_roots.back());
    auto data_roots_root = field_ct(public_witness_ct(&composer, rollup.data_roots_root));
    auto num_txs = uint32_ct(public_witness_ct(&composer, rollup.num_txs));
    auto rollup_root = field_ct(witness_ct(&composer, rollup.rollup_root));

    auto new_data_values = std::vector<byte_array_ct>();
    auto new_null_indicies = std::vector<field_ct>();
    auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    std::vector<recursion_output<bn254>> recursion_outputs(rollup_size);

    for (size_t i = 0; i < rollup_size; ++i) {
        // Verify the inner proof.
        recursion_outputs[i] = verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(
            &composer, inner_verification_key, recursive_manifest, waffle::plonk_proof{ rollup.txs[i] });

        if (can_throw && composer.failed) {
            throw std::runtime_error("Failed to verify proof: " + std::to_string(i));
        }

        // Add the proofs data values to the list. If this is a noop proof (padding), then the data values are zeros.
        // TODO: i should be able to be a constant, but causes things to fail :/
        auto is_real = num_txs > uint32_ct(witness_ct(&composer, i));
        auto public_inputs = recursion_outputs[i].public_inputs;
        new_data_values.push_back(
            byte_array_ct(&composer).write(public_inputs[2] * is_real).write(public_inputs[3] * is_real));
        new_data_values.push_back(
            byte_array_ct(&composer).write(public_inputs[4] * is_real).write(public_inputs[5] * is_real));

        // Check this proofs data root exists in the data root tree (unless a padding entry).
        auto data_root = public_inputs[6];
        auto data_roots_path = create_witness_hash_path(composer, rollup.data_roots_paths[i]);
        auto data_root_index = uint32_ct(witness_ct(&composer, rollup.data_roots_indicies[i]));
        bool_ct valid =
            data_root_index <= rollup_id &&
            check_membership(
                composer, data_roots_root, data_roots_path, byte_array_ct(data_root), byte_array_ct(data_root_index));
        composer.assert_equal(is_real.witness_index, valid.witness_index);
        if (can_throw && composer.failed) {
            throw std::runtime_error("Data root incorrect for proof: " + std::to_string(i));
        }

        new_null_indicies.push_back(public_inputs[7]);
        new_null_indicies.push_back(public_inputs[8]);

        propagate_inner_proof_public_inputs(composer, public_inputs);
    }

    // std::cout << new_data_values[0] << std::endl;
    // std::cout << new_data_values[1] << std::endl;

    size_t height = numeric::get_msb(rollup_size) + 1;
    auto zero_subtree_root = field_ct(zero_hash_at_height(height));
    // std::cout << "height: " << height << std::endl;
    // std::cout << "zsr: " << zero_subtree_root << std::endl;
    // std::cout << "rollup_root: " << rollup_root << std::endl;
    // std::cout << "old_data_root: " << old_data_root << std::endl;
    // std::cout << "old_data_path: " << rollup.old_data_path << std::endl;
    // std::cout << "new_data_root: " << new_data_root << std::endl;
    // std::cout << "new_data_path: " << rollup.new_data_path << std::endl;
    assert_check_tree(composer, rollup_root, new_data_values);
    if (can_throw && composer.failed) {
        throw std::runtime_error("Failed subtree check.");
    }

    auto new_data_path = create_witness_hash_path(composer, rollup.new_data_path);
    auto old_data_path = create_witness_hash_path(composer, rollup.old_data_path);
    update_subtree_membership(composer,
                              new_data_root,
                              new_data_path,
                              rollup_root,
                              old_data_root,
                              old_data_path,
                              zero_subtree_root,
                              byte_array_ct(data_start_index),
                              height);
    if (can_throw && composer.failed) {
        throw std::runtime_error("Failed subtree update.");
    }

    auto new_nullifier_value = byte_array_ct(&composer, 64);
    new_nullifier_value.set_bit(511, 1);
    field_ct last_real_null_index;

    for (size_t i = 0; i < new_null_indicies.size(); ++i) {
        auto new_null_root = field_ct(witness_ct(&composer, rollup.new_null_roots[i]));
        // TODO: i should be able to be a constant, but causes things to fail :/
        auto is_real = num_txs > uint32_ct(witness_ct(&composer, i / 2));

        // This makes padding transactions act as noops.
        last_real_null_index = (new_null_indicies[i] * is_real) + (last_real_null_index * !is_real);
        auto old_nullifier_value = byte_array_ct(&composer, 64);
        old_nullifier_value.set_bit(511, !is_real);

        auto new_null_path = create_witness_hash_path(composer, rollup.new_null_paths[i]);
        auto old_null_path = create_witness_hash_path(composer, rollup.old_null_paths[i]);

        // std::cout << "old_null_root: " << old_null_root << std::endl;
        // std::cout << "new_null_root: " << new_null_root << std::endl;
        // std::cout << "old_null_path: " << old_null_path << std::endl;
        // std::cout << "new_null_path: " << new_null_path << std::endl;
        // std::cout << "index: " << last_real_null_index << std::endl;
        // std::cout << "old_value: " << old_nullifier_value << std::endl;
        // std::cout << "new_value: " << new_nullifier_value << std::endl;

        update_membership(composer,
                          new_null_root,
                          new_null_path,
                          new_nullifier_value,
                          old_null_root,
                          old_null_path,
                          old_nullifier_value,
                          byte_array_ct(last_real_null_index));
        if (can_throw && composer.failed) {
            throw std::runtime_error("Failed nullifier update: " + std::to_string(i));
        }

        old_null_root = new_null_root;
    }

    return recursion_outputs;
}

} // namespace rollup_proofs
} // namespace rollup
