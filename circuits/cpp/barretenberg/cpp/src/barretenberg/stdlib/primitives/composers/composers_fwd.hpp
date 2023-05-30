#pragma once

namespace proof_system::plonk {
class StandardPlonkComposer;
class TurboPlonkComposer;
class UltraPlonkComposer;

class StandardPlonkComposer;
} // namespace proof_system::plonk

namespace proof_system::honk {
class StandardHonkComposer;
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
    extern template class stdlib_type<plonk::UltraPlonkComposer>;

#define EXTERN_STDLIB_METHOD(stdlib_method)                                                                            \
    extern template stdlib_method(proof_system::StandardCircuitConstructor);                                           \
    extern template stdlib_method(proof_system::TurboCircuitConstructor);                                              \
    extern template stdlib_method(proof_system::UltraCircuitConstructor);                                              \
    extern template stdlib_method(plonk::StandardPlonkComposer);                                                       \
    extern template stdlib_method(honk::StandardHonkComposer);                                                         \
    extern template stdlib_method(plonk::TurboPlonkComposer);                                                          \
    extern template stdlib_method(plonk::UltraPlonkComposer);

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
    extern template class stdlib_type<plonk::UltraPlonkComposer>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<proof_system::UltraCircuitConstructor, __VA_ARGS__>;                             \
    extern template class stdlib_type<plonk::UltraPlonkComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_METHOD(stdlib_method)                                                                      \
    extern template stdlib_method(proof_system::UltraCircuitConstructor);                                              \
    extern template stdlib_method(plonk::UltraPlonkComposer);
