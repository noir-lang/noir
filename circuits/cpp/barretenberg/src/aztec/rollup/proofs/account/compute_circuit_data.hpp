#pragma once
#include "account.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include "../compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace account {

using circuit_data = proofs::circuit_data;

inline circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                     std::string const& key_path,
                                     bool compute = true,
                                     bool save = true,
                                     bool load = true)
{
    std::cerr << "Getting account circuit data..." << std::endl;
    auto name = format("account");

    auto build_circuit = [&](Composer& composer) {
        account_tx tx;
        tx.account_path.resize(DATA_TREE_DEPTH);
        account_circuit(composer, tx);
    };

    return proofs::get_circuit_data(name, srs, key_path, compute, save, load, true, true, false, build_circuit);
}

// Deprecated. Just use get_circuit_data.
inline circuit_data compute_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs)
{
    return get_circuit_data(srs, "", true, false, false);
}

// Deprecated. Just use get_circuit_data.
inline circuit_data compute_or_load_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                                 std::string const& key_path)
{
    return get_circuit_data(srs, key_path, true, true, true);
}

} // namespace account
} // namespace proofs
} // namespace rollup
