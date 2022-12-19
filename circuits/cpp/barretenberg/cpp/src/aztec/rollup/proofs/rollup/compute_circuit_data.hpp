#pragma once
#include "create_rollup_tx.hpp"
#include "rollup_circuit.hpp"
#include "../compute_circuit_data.hpp"
#include "../join_split/index.hpp"
#include "../account/index.hpp"
#include "../claim/index.hpp"
#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

struct circuit_data : proofs::circuit_data {
    size_t rollup_size;
    size_t num_txs;
    std::vector<std::shared_ptr<waffle::verification_key>> verification_keys;
    join_split::circuit_data join_split_circuit_data;
};

inline circuit_data get_circuit_data(size_t rollup_size,
                                     join_split::circuit_data const& join_split_circuit_data,
                                     account::circuit_data const& account_circuit_data,
                                     claim::circuit_data const& claim_circuit_data,
                                     std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true,
                                     bool pk = true,
                                     bool vk = true,
                                     bool mock = false)
{
    auto floor_max_txs = 1UL << numeric::get_msb(rollup_size);
    auto rollup_size_pow2 = rollup_size == floor_max_txs ? rollup_size : floor_max_txs << 1UL;
    std::cerr << "Getting tx rollup circuit data: (txs: " << rollup_size << ", size: " << rollup_size_pow2 << ")"
              << std::endl;
    auto name = "rollup_" + std::to_string(rollup_size);
    auto verification_keys = { join_split_circuit_data.verification_key, // padding
                               join_split_circuit_data.verification_key, // deposit
                               join_split_circuit_data.verification_key, // withdraw
                               join_split_circuit_data.verification_key, // send
                               account_circuit_data.verification_key,
                               join_split_circuit_data.verification_key, // defi deposit
                               claim_circuit_data.verification_key };

    auto build_circuit = [&](Composer& composer) {
        auto rollup = create_padding_rollup(rollup_size, join_split_circuit_data.padding_proof);
        rollup_circuit(composer, rollup, verification_keys, rollup_size);
    };

    auto cd =
        proofs::get_circuit_data<Composer>("tx rollup",
                                           name,
                                           srs,
                                           key_path,
                                           compute,
                                           save,
                                           load,
                                           pk,
                                           vk,
                                           true,
                                           mock,
                                           build_circuit,
                                           " " + std::to_string(rollup_size) + "x" + std::to_string(rollup_size_pow2));

    circuit_data data;
    data.num_gates = cd.num_gates;
    data.padding_proof = cd.padding_proof;
    data.proving_key = cd.proving_key;
    data.verification_key = cd.verification_key;
    data.verification_keys = verification_keys;
    data.num_txs = rollup_size;
    data.rollup_size = rollup_size_pow2;
    data.join_split_circuit_data = join_split_circuit_data;
    data.srs = cd.srs;
    data.mock = cd.mock;

    return data;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
