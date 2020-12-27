#pragma once
#include "../rollup/compute_circuit_data.hpp"
#include "../account/compute_circuit_data.hpp"
#include "root_rollup_tx.hpp"
#include "root_rollup_circuit.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace join_split;
using namespace account;

struct circuit_data : proofs::circuit_data {
    size_t num_inner_rollups;
    rollup::circuit_data inner_rollup_circuit_data;
};

inline circuit_data get_circuit_data(size_t num_inner_rollups,
                                     rollup::circuit_data const& rollup_circuit_data,
                                     std::string const& srs_path,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true)
{
    std::cerr << "Getting root rollup circuit data: (size: " << num_inner_rollups * rollup_circuit_data.rollup_size
              << ")" << std::endl;
    auto name = "root_rollup_" + std::to_string(num_inner_rollups);

    auto build_circuit = [&](Composer& composer) {
        auto gibberish_roots_path =
            fr_hash_path(ROOT_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

        root_rollup_tx root_rollup;
        root_rollup.rollup_id = 0;
        root_rollup.old_data_roots_root = fr::random_element();
        root_rollup.new_data_roots_root = fr::random_element();
        root_rollup.old_data_roots_path = gibberish_roots_path;
        root_rollup.new_data_roots_path = gibberish_roots_path;
        root_rollup.rollups.resize(num_inner_rollups, rollup_circuit_data.padding_proof);
        root_rollup_circuit(
            composer, root_rollup, rollup_circuit_data.rollup_size, rollup_circuit_data.verification_key);
    };

    auto cd = proofs::get_circuit_data(name, srs_path, key_path, compute, save, load, build_circuit);

    circuit_data data;
    data.num_gates = cd.num_gates;
    data.padding_proof = cd.padding_proof;
    data.proving_key = cd.proving_key;
    data.verification_key = cd.verification_key;
    data.num_inner_rollups = num_inner_rollups;
    data.inner_rollup_circuit_data = rollup_circuit_data;

    return data;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
