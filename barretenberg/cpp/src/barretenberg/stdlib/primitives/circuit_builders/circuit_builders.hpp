/**
 * @brief Contains all the headers required to adequately compile the types defined in circuit_builders_fwd.hpp and
 * instantiate templates.
 */
#pragma once
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

template <typename T>
concept HasPlookup =
    proof_system::IsAnyOf<T, proof_system::UltraCircuitBuilder, proof_system::GoblinUltraCircuitBuilder>;

template <typename T>
concept IsGoblinBuilder = proof_system::IsAnyOf<T, proof_system::GoblinUltraCircuitBuilder>;

#define INSTANTIATE_STDLIB_METHOD(stdlib_method)                                                                       \
    template stdlib_method(proof_system::StandardCircuitBuilder);                                                      \
    template stdlib_method(proof_system::UltraCircuitBuilder);                                                         \
    template stdlib_method(proof_system::GoblinUltraCircuitBuilder);

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<proof_system::StandardCircuitBuilder>;                                                  \
    template class stdlib_type<proof_system::UltraCircuitBuilder>;                                                     \
    template class stdlib_type<proof_system::GoblinUltraCircuitBuilder>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<proof_system::StandardCircuitBuilder, __VA_ARGS__>;                                     \
    template class stdlib_type<proof_system::UltraCircuitBuilder, __VA_ARGS__>;                                        \
    template class stdlib_type<proof_system::GoblinUltraCircuitBuilder, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_BASIC_TYPE(stdlib_type) template class stdlib_type<proof_system::StandardCircuitBuilder>;

#define INSTANTIATE_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::StandardCircuitBuilder, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_ULTRA_METHOD(stdlib_method)                                                                 \
    template stdlib_method(proof_system::UltraCircuitBuilder);                                                         \
    template stdlib_method(proof_system::GoblinUltraCircuitBuilder);

#define INSTANTIATE_STDLIB_ULTRA_TYPE(stdlib_type)                                                                     \
    template class stdlib_type<proof_system::UltraCircuitBuilder>;                                                     \
    template class stdlib_type<proof_system::GoblinUltraCircuitBuilder>;

#define INSTANTIATE_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::UltraCircuitBuilder, __VA_ARGS__>;                                        \
    template class stdlib_type<proof_system::GoblinUltraCircuitBuilder, __VA_ARGS__>;
