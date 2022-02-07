#pragma once
#include "account.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include "../compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace account {

using circuit_data = proofs::circuit_data;

inline circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs, bool mock = false)
{
    std::cerr << "Getting account circuit data..." << std::endl;

    auto build_circuit = [&](Composer& composer) {
        account_tx tx;
        tx.account_path.resize(DATA_TREE_DEPTH);
        account_circuit(composer, tx);
    };

    return proofs::get_circuit_data<Composer>(
        "account", "", srs, "", true, false, false, true, true, false, mock, build_circuit);
}

} // namespace account
} // namespace proofs
} // namespace rollup
