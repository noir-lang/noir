#pragma once
#include <cstdint>

enum ComposerType : uint64_t {
    Standard = 1 << 0,
    Turbo = 1 << 1,
};
