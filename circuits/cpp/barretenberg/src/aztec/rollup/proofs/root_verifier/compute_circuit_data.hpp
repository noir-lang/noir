#pragma once
#include "../root_rollup/compute_circuit_data.hpp"
#include "root_verifier_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

struct circuit_data : proofs::circuit_data {
    std::vector<std::shared_ptr<waffle::verification_key>> valid_vks;
};

inline circuit_data get_circuit_data(root_rollup::circuit_data const& root_rollup_circuit_data,
                                     std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                     std::vector<std::shared_ptr<waffle::verification_key>> const& valid_vks,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true,
                                     bool pk = true,
                                     bool vk = true)
{
    std::cerr << "Getting root verifier circuit data: (size: " << root_rollup_circuit_data.rollup_size << ")"
              << std::endl;
    auto name =
        format("root_verifier_", root_rollup_circuit_data.inner_rollup_circuit_data.rollup_size, "_", valid_vks.size());

    auto build_verifier_circuit = [&](OuterComposer& composer) {
        root_verifier_tx tx;
        tx.proof_data = root_rollup_circuit_data.padding_proof;
        root_verifier_circuit(composer, tx, root_rollup_circuit_data.verification_key, valid_vks);
    };

    auto cd = proofs::get_circuit_data<OuterComposer>(
        name, srs, key_path, compute, save, load, pk, vk, false, build_verifier_circuit);

    circuit_data data;
    data.num_gates = cd.num_gates;
    data.verifier_crs = cd.verifier_crs;
    data.proving_key = cd.proving_key;
    data.verification_key = cd.verification_key;
    data.valid_vks = valid_vks;

    return data;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
