#pragma once
#include <concepts>
#include <cstdint>

namespace bb {
// TODO(#731): Changing the explicit value of these enum elements breaks brittle and outdated tests in circuits/cpp.
enum class CircuitType : uint32_t { STANDARD = 0, ULTRA = 2, UNDEFINED = 3 };

template <typename T, typename... U>
concept IsAnyOf = (std::same_as<T, U> || ...);
} // namespace bb