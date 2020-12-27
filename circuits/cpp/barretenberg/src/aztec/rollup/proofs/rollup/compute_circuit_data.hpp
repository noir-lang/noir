#pragma once
#include "create_rollup.hpp"
#include "rollup_circuit.hpp"
#include "../compute_circuit_data.hpp"
#include "../join_split/compute_circuit_data.hpp"
#include "../account/compute_circuit_data.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

struct circuit_data : proofs::circuit_data {
    size_t rollup_size;
    std::vector<std::shared_ptr<waffle::verification_key>> verification_keys;
    join_split::circuit_data join_split_circuit_data;
};

inline circuit_data get_circuit_data(size_t rollup_size,
                                     join_split::circuit_data const& join_split_circuit_data,
                                     account::circuit_data const& account_circuit_data,
                                     std::string const& srs_path,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true)
{
    auto floor_max_txs = 1UL << numeric::get_msb(rollup_size);
    auto rollup_size_pow2 = rollup_size == floor_max_txs ? rollup_size : floor_max_txs << 1UL;
    std::cerr << "Getting rollup circuit data: (size: " << rollup_size << ", actual: " << rollup_size_pow2 << ")"
              << std::endl;
    auto name = "rollup_" + std::to_string(rollup_size);
    auto verification_keys = { join_split_circuit_data.verification_key, account_circuit_data.verification_key };

    auto build_circuit = [&](Composer& composer) {
        auto rollup = create_padding_rollup(rollup_size, join_split_circuit_data.padding_proof);
        rollup_circuit(composer, rollup, verification_keys, rollup_size);
    };

    auto cd = proofs::get_circuit_data(name, srs_path, key_path, compute, save, load, build_circuit);

    circuit_data data;
    data.num_gates = cd.num_gates;
    data.padding_proof = cd.padding_proof;
    data.proving_key = cd.proving_key;
    data.verification_key = cd.verification_key;
    data.verification_keys = verification_keys;
    data.rollup_size = rollup_size;
    data.join_split_circuit_data = join_split_circuit_data;

    return data;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
