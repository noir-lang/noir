#pragma once
#include "../rollup/compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

struct circuit_data : proofs::circuit_data {
    size_t num_inner_rollups;
    size_t rollup_size;
    rollup::circuit_data inner_rollup_circuit_data;
};

circuit_data get_circuit_data(size_t num_inner_rollups,
                              rollup::circuit_data const& rollup_circuit_data,
                              std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                              std::string const& key_path,
                              bool compute = true,
                              bool save = true,
                              bool load = true,
                              bool pk = true,
                              bool vk = true,
                              bool mock = false);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
