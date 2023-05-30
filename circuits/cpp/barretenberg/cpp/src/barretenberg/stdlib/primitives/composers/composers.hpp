#pragma once
#include "barretenberg/plonk/composer/standard_plonk_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/plonk/composer/standard_plonk_composer.hpp"
#include "barretenberg/plonk/composer/turbo_plonk_composer.hpp"
#include "barretenberg/plonk/composer/ultra_plonk_composer.hpp"
#include "barretenberg/honk/composer/ultra_honk_composer.hpp"
#include "barretenberg/proof_system/circuit_constructors/standard_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/turbo_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"

#define INSTANTIATE_STDLIB_METHOD(stdlib_method)                                                                       \
    template stdlib_method(proof_system::StandardCircuitConstructor);                                                  \
    template stdlib_method(proof_system::TurboCircuitConstructor);                                                     \
    template stdlib_method(proof_system::UltraCircuitConstructor);                                                     \
    template stdlib_method(plonk::StandardPlonkComposer);                                                              \
    template stdlib_method(honk::StandardHonkComposer);                                                                \
    template stdlib_method(plonk::TurboPlonkComposer);                                                                 \
    template stdlib_method(plonk::UltraPlonkComposer);

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<proof_system::StandardCircuitConstructor>;                                              \
    template class stdlib_type<plonk::StandardPlonkComposer>;                                                          \
    template class stdlib_type<honk::StandardHonkComposer>;                                                            \
    template class stdlib_type<proof_system::TurboCircuitConstructor>;                                                 \
    template class stdlib_type<plonk::TurboPlonkComposer>;                                                             \
    template class stdlib_type<proof_system::UltraCircuitConstructor>;                                                 \
    template class stdlib_type<plonk::UltraPlonkComposer>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                                 \
    template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                             \
    template class stdlib_type<honk::StandardPlonkComposer, __VA_ARGS__>;                                              \
    template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                                    \
    template class stdlib_type<plonk::TurboPlonkComposer, __VA_ARGS__>;                                                \
    template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;                                    \
    template class stdlib_type<plonk::UltraPlonkComposer, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_BASIC_TYPE(stdlib_type)                                                                     \
    template class stdlib_type<proof_system::StandardCircuitConstructor>;                                              \
    template class stdlib_type<plonk::StandardPlonkComposer>;                                                          \
    template class stdlib_type<honk::StandardPlonkComposer>;                                                           \
    template class stdlib_type<proof_system::TurboCircuitConstructor>;                                                 \
    template class stdlib_type<plonk::TurboPlonkComposer>;

#define INSTANTIATE_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::StandardCircuitConstructor, __VA_ARGS__>;                                 \
    template class stdlib_type<honk::StandardHonkComposer, __VA_ARGS__>;                                               \
    template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                             \
    template class stdlib_type<proof_system::TurboCircuitConstructor, __VA_ARGS__>;                                    \
    template class stdlib_type<plonk::TurboPlonkComposer, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_ULTRA_METHOD(stdlib_method)                                                                 \
    template stdlib_method(proof_system::UltraCircuitConstructor);                                                     \
    template stdlib_method(plonk::UltraPlonkComposer);

#define INSTANTIATE_STDLIB_ULTRA_TYPE(stdlib_type)                                                                     \
    template class stdlib_type<proof_system::UltraCircuitConstructor>;                                                 \
    template class stdlib_type<plonk::UltraPlonkComposer>;

#define INSTANTIATE_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;                                    \
    template class stdlib_type<plonk::UltraPlonkComposer, __VA_ARGS__>;
