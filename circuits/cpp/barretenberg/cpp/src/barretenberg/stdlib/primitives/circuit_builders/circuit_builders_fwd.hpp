/**
 * @brief Defines particular composer and circuit constructor types expected to be used for proof or circuit
construction in stdlib and contains macros for explicit instantiation.
 *
 * @details This file is designed to be included in header files to instruct the compiler that these classes exist and
 * their instantiation will eventually take place. Given it has no dependencies, it causes no additional compilation or
 *  propagation.
 */
#pragma once
#include <concepts>

namespace proof_system::honk {
namespace flavor {
class Standard;
class Ultra;
} // namespace flavor
} // namespace proof_system::honk

namespace proof_system {
class StandardCircuitBuilder;
class TurboCircuitBuilder;
class UltraCircuitBuilder;
} // namespace proof_system

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<proof_system::StandardCircuitBuilder>;                                           \
    extern template class stdlib_type<proof_system::TurboCircuitBuilder>;                                              \
    extern template class stdlib_type<proof_system::UltraCircuitBuilder>;

#define EXTERN_STDLIB_METHOD(stdlib_method)                                                                            \
    extern template stdlib_method(proof_system::StandardCircuitBuilder);                                               \
    extern template stdlib_method(proof_system::TurboCircuitBuilder);                                                  \
    extern template stdlib_method(proof_system::UltraCircuitBuilder);

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<proof_system::StandardCircuitBuilder, __VA_ARGS__>;                              \
    extern template class stdlib_type<proof_system::TurboCircuitBuilder, __VA_ARGS__>;                                 \
    extern template class stdlib_type<proof_system::UltraCircuitBuilder, __VA_ARGS__>;

#define EXTERN_STDLIB_BASIC_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<proof_system::StandardCircuitBuilder>;                                           \
    extern template class stdlib_type<proof_system::TurboCircuitBuilder>;

#define EXTERN_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::StandardCircuitBuilder, __VA_ARGS__>;                              \
    extern template class stdlib_type<proof_system::TurboCircuitBuilder, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_TYPE(stdlib_type) extern template class stdlib_type<proof_system::UltraCircuitBuilder>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::UltraCircuitBuilder, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_METHOD(stdlib_method) extern template stdlib_method(proof_system::UltraCircuitBuilder);
