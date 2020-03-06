#pragma once

#include <cstdint>

namespace waffle {
class settings_base {
  public:
    static constexpr bool requires_shifted_wire(const uint64_t wire_shift_settings, const uint64_t wire_index)
    {
        return (((wire_shift_settings >> (wire_index)) & 1UL) == 1UL);
    }
};

class standard_settings : public settings_base {
  public:
    static constexpr size_t program_width = 3;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr bool uses_quotient_mid = true;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
};

class turbo_settings : public settings_base {
  public:
    static constexpr size_t program_width = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
};
} // namespace waffle