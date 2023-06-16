/**
 * @brief Defines particular composer and circuit constructor types expected to be used for proof or circuit
construction in stdlib and contains macros for explicit instantiation.
 *
 * @details This file is designed to be included in header files to instruct the compiler that these classes exist and
 * their instantiation will eventually take place. Given it has no dependencies, it causes no additional compilation or
 *  propagation.
 */
#pragma once

namespace proof_system::plonk {
class StandardPlonkComposer;
class TurboPlonkComposer;
class UltraPlonkComposer;

} // namespace proof_system::plonk

namespace proof_system::honk {
namespace flavor {
class Standard;
class Ultra;
} // namespace flavor
template <class Flavor> class StandardHonkComposer_;
using StandardHonkComposer = StandardHonkComposer_<flavor::Standard>;
template <class Flavor> class UltraHonkComposer_;
using UltraHonkComposer = UltraHonkComposer_<flavor::Ultra>;
} // namespace proof_system::honk

namespace proof_system {
class StandardCircuitConstructor;
class TurboCircuitConstructor;
class UltraCircuitConstructor;
} // namespace proof_system

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor>;                                       \
    extern template class stdlib_type<plonk::StandardPlonkComposer>;                                                   \
    extern template class stdlib_type<honk::StandardHonkComposer>;                                                     \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor>;                                          \
    extern template class stdlib_type<plonk::TurboPlonkComposer>;                                                      \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor>;                                          \
    extern template class stdlib_type<plonk::UltraPlonkComposer>;                                                      \
    extern template class stdlib_type<honk::UltraHonkComposer>;

#define EXTERN_STDLIB_METHOD(stdlib_method)                                                                            \
    extern template stdlib_method(proof_system::StandardCircuitConstructor);                                           \
    extern template stdlib_method(proof_system::TurboCircuitConstructor);                                              \
    extern template stdlib_method(proof_system::UltraCircuitConstructor);                                              \
    extern template stdlib_method(plonk::StandardPlonkComposer);                                                       \
    extern template stdlib_method(honk::StandardHonkComposer);                                                         \
    extern template stdlib_method(plonk::TurboPlonkComposer);                                                          \
    extern template stdlib_method(plonk::UltraPlonkComposer);                                                          \
    extern template stdlib_method(honk::UltraHonkComposer);

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                          \
    extern template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                      \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<plonk::TurboPlonkComposer, __VA_ARGS__>;                                         \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<plonk::UltraPlonkComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_BASIC_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor>;                                       \
    extern template class stdlib_type<plonk::StandardPlonkComposer>;                                                   \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor>;                                          \
    extern template class stdlib_type<plonk::TurboPlonkComposer>;

#define EXTERN_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                          \
    extern template class stdlib_type<honk::StandardHonkComposer, __VA_ARGS__>;                                        \
    extern template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                      \
    extern template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<plonk::TurboPlonkComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor>;                                          \
    extern template class stdlib_type<plonk::UltraPlonkComposer>;                                                      \
    extern template class stdlib_type<honk::UltraHonkComposer>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<plonk::UltraPlonkComposer, __VA_ARGS__>;                                         \
    extern template class stdlib_type<honk::UltraHonkComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_METHOD(stdlib_method)                                                                      \
    extern template stdlib_method(proof_system::UltraCircuitConstructor);                                              \
    extern template stdlib_method(plonk::UltraPlonkComposer);                                                          \
    extern template stdlib_method(honk::UltraHonkComposer);
