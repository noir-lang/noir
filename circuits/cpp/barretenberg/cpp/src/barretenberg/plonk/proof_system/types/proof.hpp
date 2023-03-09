#pragma once
#include <cstdint>
#include <vector>

namespace plonk {

struct proof {
    std::vector<uint8_t> proof_data;
};

} // namespace plonk