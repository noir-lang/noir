#pragma once
#include <cstdint>
#include <concepts>

namespace proof_system {
enum class CircuitType : uint32_t { STANDARD, TURBO, ULTRA, UNDEFINED };

template <typename T, typename... U> concept IsAnyOf = (std::same_as<T, U> || ...);
} // namespace proof_system