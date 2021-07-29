#pragma once
#include "../rollup/compute_circuit_data.hpp"
#include "root_rollup_tx.hpp"
#include "root_rollup_circuit.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace join_split;

struct circuit_data : proofs::circuit_data {
    size_t num_inner_rollups;
    size_t rollup_size;
    rollup::circuit_data inner_rollup_circuit_data;
};

inline circuit_data get_circuit_data(size_t num_inner_rollups,
                                     rollup::circuit_data const& rollup_circuit_data,
                                     std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true,
                                     bool pk = true,
                                     bool vk = true)
{
    auto rollup_size = num_inner_rollups * rollup_circuit_data.rollup_size;
    auto floor = 1UL << numeric::get_msb(rollup_size);
    auto rollup_size_pow2 = rollup_size == floor ? rollup_size : floor << 1UL;
    std::cerr << "Getting root rollup circuit data: (size: " << rollup_size_pow2 << ")" << std::endl;
    auto name = format("root_rollup_", rollup_circuit_data.num_txs, "x", num_inner_rollups);

    auto build_circuit = [&](Composer& composer) {
        auto gibberish_roots_path =
            fr_hash_path(ROOT_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));
        auto gibberish_defi_path =
            fr_hash_path(DEFI_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

        root_rollup_tx root_rollup;
        root_rollup.old_data_roots_root = fr::random_element();
        root_rollup.new_data_roots_root = fr::random_element();
        root_rollup.old_data_roots_path = gibberish_roots_path;
        root_rollup.rollups.resize(num_inner_rollups, rollup_circuit_data.padding_proof);
        root_rollup.old_defi_root = fr::random_element();
        root_rollup.new_defi_root = fr::random_element();
        root_rollup.old_defi_path = gibberish_defi_path;
        root_rollup.bridge_ids.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
        root_rollup.asset_ids.resize(NUM_ASSETS, MAX_NUM_ASSETS);
        root_rollup.defi_interaction_notes.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
        root_rollup.num_previous_defi_interactions = 0;
        root_rollup_circuit(composer,
                            root_rollup,
                            rollup_circuit_data.rollup_size,
                            rollup_size_pow2,
                            rollup_circuit_data.verification_key);
    };

    auto cd = proofs::get_circuit_data(name, srs, key_path, compute, save, load, pk, vk, false, build_circuit);

    circuit_data data;
    data.num_gates = cd.num_gates;
    data.verifier_crs = cd.verifier_crs;
    data.padding_proof = cd.padding_proof;
    data.proving_key = cd.proving_key;
    data.verification_key = cd.verification_key;
    data.num_inner_rollups = num_inner_rollups;
    data.rollup_size = rollup_size_pow2;
    data.inner_rollup_circuit_data = rollup_circuit_data;

    return data;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
