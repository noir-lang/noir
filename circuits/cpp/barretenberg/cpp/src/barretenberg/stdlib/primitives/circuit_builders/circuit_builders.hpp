/**
 * @brief Contains all the headers required to adequately compile the types defined in composers_fwd.hpp and instantiate
 * templates.
 */
#pragma once
#include "barretenberg/proof_system/circuit_constructors/standard_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/turbo_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"

#define INSTANTIATE_STDLIB_METHOD(stdlib_method)                                                                       \
    template stdlib_method(proof_system::StandardCircuitConstructor);                                                  \
    template stdlib_method(proof_system::TurboCircuitConstructor);                                                     \
    template stdlib_method(proof_system::UltraCircuitConstructor);

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<proof_system::StandardCircuitConstructor>;                                              \
    template class stdlib_type<proof_system::TurboCircuitConstructor>;                                                 \
    template class stdlib_type<proof_system::UltraCircuitConstructor>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                                 \
    template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                                    \
    template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_BASIC_TYPE(stdlib_type)                                                                     \
    template class stdlib_type<proof_system::StandardCircuitConstructor>;                                              \
    template class stdlib_type<proof_system::TurboCircuitConstructor>;

#define INSTANTIATE_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                                 \
    template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_ULTRA_METHOD(stdlib_method) template stdlib_method(proof_system::UltraCircuitConstructor);

#define INSTANTIATE_STDLIB_ULTRA_TYPE(stdlib_type) template class stdlib_type<proof_system::UltraCircuitConstructor>;

#define INSTANTIATE_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;
