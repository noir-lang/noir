
#pragma once
#include <cstdint>
#include <vector>

namespace proof_system::plonk {

struct commitment_open_proof {
    std::vector<uint8_t> proof_data;
};

} // namespace proof_system::plonk
