#include "rollup_circuit.hpp"
#include <stdlib/merkle_tree/membership.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;

fr zero_hash_at_height(size_t height)
{
    auto current = hash_value_native(std::vector<uint8_t>(64, 0));
    for (size_t i = 0; i < height; ++i) {
        current = compress_native({ current, current });
    }
    return current;
}

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

std::vector<recursion_output<field_ct, group_ct>> rollup_circuit(
    Composer& composer,
    rollup_tx const& rollup,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    size_t rollup_size)
{
    auto data_start_index = field_ct(public_witness_ct(&composer, rollup.data_start_index));
    auto old_data_root = field_ct(public_witness_ct(&composer, rollup.old_data_root));
    auto new_data_root = field_ct(public_witness_ct(&composer, rollup.new_data_root));
    auto rollup_root = field_ct(witness_ct(&composer, rollup.rollup_root));

    auto num_txs = uint32_ct(witness_ct(&composer, rollup.num_txs));
    auto new_data_values = std::vector<byte_array_ct>();
    auto new_null_indicies = std::vector<byte_array_ct>();
    auto recursive_manifest = Composer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    std::vector<recursion_output<field_ct, group_ct>> recursion_outputs(rollup_size);

    for (size_t i = 0; i < rollup_size; ++i) {
        // Verify the inner proof.
        recursion_outputs[i] = verify_proof<Composer, recursive_turbo_verifier_settings>(
            &composer, inner_verification_key, recursive_manifest, { rollup.txs[i] });

        // Add the proofs data values to the list. If this is a noop proof (padding), then the data values are zeros.
        // TODO: i should be able to be a constant, but causes things to fail :/
        auto is_real = num_txs > uint32_ct(witness_ct(&composer, i));
        auto public_inputs = recursion_outputs[i].public_inputs;
        new_data_values.push_back(
            byte_array_ct(&composer).write(public_inputs[2] * is_real).write(public_inputs[3] * is_real));
        new_data_values.push_back(
            byte_array_ct(&composer).write(public_inputs[4] * is_real).write(public_inputs[5] * is_real));

        // Check this proofs old data root is equal to the one we've been given.
        composer.assert_equal(old_data_root.witness_index, public_inputs[6].witness_index);

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

    auto old_null_root = field_ct(public_witness_ct(&composer, rollup.old_null_root));
    for (size_t i = 0; i < new_null_indicies.size(); ++i) {
        auto new_null_root = field_ct(witness_ct(&composer, rollup.new_null_roots[i]));
        // TODO: i should be able to be a constant, but causes things to fail :/
        auto is_real = num_txs > uint32_ct(witness_ct(&composer, i/2));
        auto nullifier_value = byte_array_ct(&composer, 64);
        nullifier_value.set_bit(511, is_real);

        auto new_null_path = create_witness_hash_path(composer, rollup.new_null_paths[i]);
        auto old_null_path = create_witness_hash_path(composer, rollup.old_null_paths[i]);

        // std::cout << "old_null_root: " << old_null_root << std::endl;
        // std::cout << "new_null_root: " << new_null_root << std::endl;
        // std::cout << "old_null_path: " << old_null_path << std::endl;
        // std::cout << "new_null_path: " << new_null_path << std::endl;
        // std::cout << "index: " << new_null_indicies[i] << std::endl;
        // std::cout << "value: " << nullifier_value << std::endl;

        update_membership(composer,
                          new_null_root,
                          new_null_path,
                          nullifier_value,
                          old_null_root,
                          old_null_path,
                          byte_array_ct(&composer, 64),
                          new_null_indicies[i]);

        old_null_root = new_null_root;
    }

    // Make the latest null root public.
    composer.set_public_input(old_null_root.witness_index);

    return recursion_outputs;
}

} // namespace rollup_proofs
} // namespace rollup
