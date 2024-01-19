
#pragma once
#include <cstdint>
#include <vector>

namespace bb::plonk {

struct commitment_open_proof {
    std::vector<uint8_t> proof_data;
};

} // namespace bb::plonk
