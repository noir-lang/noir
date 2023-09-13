#pragma once
#include <cstdint>

enum CircuitType : uint64_t {
    Standard = 1 << 0,
    Turbo = 1 << 1,
};
