#pragma once
#include <stdlib/types/turbo.hpp>
#include "../rollup/rollup_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;

struct root_rollup_proof_data {
    fr input_hash;
    g1::affine_element recursion_output[2];

    root_rollup_proof_data(std::vector<uint8_t> const& proof_data);
    root_rollup_proof_data(std::vector<fr> const& public_inputs);

    bool operator==(const root_rollup_proof_data& other) const = default;

  private:
    void populate_from_fields(std::vector<fr> const& fields);
};

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
