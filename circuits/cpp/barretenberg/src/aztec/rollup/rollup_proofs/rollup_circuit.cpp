#include "rollup_circuit.hpp"
#include <stdlib/merkle_tree/membership.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;
using namespace plonk::stdlib::merkle_tree;

std::vector<recursion_output<field_ct, group_ct>> rollup_circuit(
    Composer& composer,
    rollup_tx const& rollup,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    size_t rollup_size)
{
    auto num_public_inputs = inner_verification_key->num_public_inputs;

    // Do annoying transform from raw bytes to waffle::plonk_proof.
    std::vector<waffle::plonk_proof> proofs(rollup.txs.size());
    std::transform(
        rollup.txs.begin(), rollup.txs.end(), proofs.begin(), [](auto const& p) { return waffle::plonk_proof{ p }; });

    auto recursive_manifest = Composer::create_unrolled_manifest(num_public_inputs);
    std::vector<recursion_output<field_ct, group_ct>> recursion_outputs(proofs.size());

    for (size_t i = 0; i < rollup_size; ++i) {
        auto output = verify_proof<Composer, recursive_turbo_verifier_settings>(
            &composer, inner_verification_key, recursive_manifest, proofs[i]);
        recursion_outputs[i] = output;
    }

    auto num_txs = uint32_ct(public_witness_ct(&composer, rollup.num_txs));
    auto dataStartIndex = field_ct(public_witness_ct(&composer, rollup.data_start_index));
    auto old_data_root = field_ct(public_witness_ct(&composer, rollup.old_data_root));
    auto old_null_root = field_ct(public_witness_ct(&composer, rollup.old_null_root));
    auto new_data_root = field_ct(public_witness_ct(&composer, rollup.new_data_root));
    auto new_null_root = field_ct(public_witness_ct(&composer, rollup.new_null_root));

    byte_array_ct nullifier_value(&composer);
    nullifier_value.write(field_ct(0)).write(field_ct(1));

    for (size_t i = 0; i < rollup_size; ++i) {
        auto is_real = num_txs < i;
        auto public_inputs = recursion_outputs[i].public_inputs;
        auto nextDataIndex = dataStartIndex + (i * 2);
        auto newData1 = byte_array_ct(&composer).write(field_ct(public_inputs[2])).write(field_ct(public_inputs[3]));
        auto newData2 = byte_array_ct(&composer).write(field_ct(public_inputs[4])).write(field_ct(public_inputs[5]));
        auto oldPath1 = create_witness_hash_path(composer, rollup.old_data_paths[i * 2].second);
        auto oldPath2 = create_witness_hash_path(composer, rollup.old_data_paths[i * 2 + 1].second);
        auto newPath1 = create_witness_hash_path(composer, rollup.new_data_paths[i * 2].second);
        auto newPath2 = create_witness_hash_path(composer, rollup.new_data_paths[i * 2 + 1].second);

        // CONDITIONALLY make newData1 and newData2 be 64 0 bytes if this is a noop.
        // newData1 =(newData1 * is_real) + (oldData * is_real)

        update_membership(composer,
                          new_data_root,
                          newPath1,
                          newData1,
                          old_data_root,
                          oldPath1,
                          byte_array_ct(&composer, 64),
                          byte_array_ct(nextDataIndex));

        update_membership(composer,
                          new_data_root,
                          newPath2,
                          newData2,
                          old_data_root,
                          oldPath2,
                          byte_array_ct(&composer, 64),
                          byte_array_ct(nextDataIndex + 1));

        auto oldNullPath1 = create_witness_hash_path(composer, rollup.old_null_paths[i * 2].second);
        auto oldNullPath2 = create_witness_hash_path(composer, rollup.old_null_paths[i * 2 + 1].second);
        auto newNullPath1 = create_witness_hash_path(composer, rollup.new_null_paths[i * 2].second);
        auto newNullPath2 = create_witness_hash_path(composer, rollup.new_null_paths[i * 2 + 1].second);
        auto nullifier1 = byte_array_ct(public_inputs[7]);
        auto nullifier2 = byte_array_ct(public_inputs[8]);

        // CONDITIONALLY make nullifier_value be 64 0 bytes if this is a noop.

        update_membership(composer,
                          new_null_root,
                          newNullPath1,
                          nullifier_value,
                          old_null_root,
                          oldNullPath1,
                          byte_array_ct(&composer, 64),
                          nullifier1);

        update_membership(composer,
                          new_null_root,
                          newNullPath2,
                          nullifier_value,
                          old_null_root,
                          oldNullPath2,
                          byte_array_ct(&composer, 64),
                          nullifier2);
    }

    return recursion_outputs;
}

} // namespace rollup_proofs
} // namespace rollup
