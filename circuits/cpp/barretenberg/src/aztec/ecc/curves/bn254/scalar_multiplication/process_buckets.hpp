#pragma once

#include <cstddef>
#include <cstdint>

namespace barretenberg {
namespace scalar_multiplication {
void radix_sort(uint64_t* keys, const size_t num_entries, const uint32_t shift) noexcept;

void process_buckets(uint64_t* wnaf_entries, const size_t num_entries, const uint32_t num_bits) noexcept;
} // namespace scalar_multiplication
} // namespace barretenberg