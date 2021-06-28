#pragma once
#include "join_split_tx.hpp"
#include "../compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

join_split_tx noop_tx();

using circuit_data = proofs::circuit_data;

circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                              std::string const& key_path,
                              bool compute = true,
                              bool save = true,
                              bool load = true);

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

} // namespace join_split
} // namespace proofs
} // namespace rollup