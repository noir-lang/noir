#pragma once
#include <cstdint>

namespace waffle {

// limb size when simulating a non-native field using bigfield class
// (needs to be a universal constant to be used by native verifier)
static constexpr uint64_t NUM_LIMB_BITS_IN_FIELD_SIMULATION = 68;
} // namespace waffle
