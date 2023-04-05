#pragma once

namespace proof_system::plonk {
class StandardComposer;
class TurboComposer;
class UltraComposer;

class StandardPlonkComposer;
} // namespace proof_system::plonk

namespace proof_system::honk {
class StandardHonkComposer;
} // namespace proof_system::honk

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<plonk::StandardComposer>;                                                        \
    extern template class stdlib_type<honk::StandardHonkComposer>;                                                     \
    extern template class stdlib_type<plonk::StandardPlonkComposer>;                                                   \
    extern template class stdlib_type<plonk::TurboComposer>;                                                           \
    extern template class stdlib_type<plonk::UltraComposer>;

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<plonk::StandardComposer, __VA_ARGS__>;                                           \
    extern template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                      \
    extern template class stdlib_type<plonk::TurboComposer, __VA_ARGS__>;                                              \
    extern template class stdlib_type<plonk::UltraComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_BASIC_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<plonk::StandardComposer>;                                                        \
    extern template class stdlib_type<plonk::StandardPlonkComposer>;                                                   \
    extern template class stdlib_type<plonk::TurboComposer>;

#define EXTERN_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<honk::StandardHonkComposer, __VA_ARGS__>;                                        \
    extern template class stdlib_type<plonk::StandardPlonkComposer, __VA_ARGS__>;                                      \
    extern template class stdlib_type<plonk::StandardComposer, __VA_ARGS__>;                                           \
    extern template class stdlib_type<plonk::TurboComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_TYPE(stdlib_type) extern template class stdlib_type<plonk::UltraComposer>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<plonk::UltraComposer, __VA_ARGS__>;
