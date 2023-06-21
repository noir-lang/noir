/**
 * @brief Defines particular composer and circuit constructor types expected to be used for proof or circuit
construction in stdlib and contains macros for explicit instantiation.
 *
 * @details This file is designed to be included in header files to instruct the compiler that these classes exist and
 * their instantiation will eventually take place. Given it has no dependencies, it causes no additional compilation or
 *  propagation.
 */
#pragma once

namespace proof_system::honk {
namespace flavor {
class Standard;
class Ultra;
} // namespace flavor
} // namespace proof_system::honk

namespace proof_system {
class StandardCircuitConstructor;
class TurboCircuitConstructor;
class UltraCircuitConstructor;
} // namespace proof_system

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor>;                                       \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor>;                                          \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor>;

#define EXTERN_STDLIB_METHOD(stdlib_method)                                                                            \
    extern template stdlib_method(proof_system::StandardCircuitConstructor);                                           \
    extern template stdlib_method(proof_system::TurboCircuitConstructor);                                              \
    extern template stdlib_method(proof_system::UltraCircuitConstructor);

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                          \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;

#define EXTERN_STDLIB_BASIC_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor>;                                       \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor>;

#define EXTERN_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                          \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_TYPE(stdlib_type) extern template class stdlib_type<proof_system::UltraCircuitConstructor>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_METHOD(stdlib_method) extern template stdlib_method(proof_system::UltraCircuitConstructor);
