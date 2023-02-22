#pragma once
#include <honk/composer/standard_honk_composer.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<honk::StandardHonkComposer>;                                                            \
    template class stdlib_type<plonk::StandardComposer>;                                                               \
    template class stdlib_type<plonk::TurboComposer>;                                                                  \
    template class stdlib_type<plonk::UltraComposer>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<plonk::StandardComposer, __VA_ARGS__>;                                                  \
    template class stdlib_type<plonk::TurboComposer, __VA_ARGS__>;                                                     \
    template class stdlib_type<plonk::UltraComposer, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_BASIC_TYPE(stdlib_type)                                                                     \
    template class stdlib_type<plonk::StandardComposer>;                                                               \
    template class stdlib_type<plonk::TurboComposer>;

#define INSTANTIATE_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<honk::StandardHonkComposer, __VA_ARGS__>;                                               \
    template class stdlib_type<plonk::StandardComposer, __VA_ARGS__>;                                                  \
    template class stdlib_type<plonk::TurboComposer, __VA_ARGS__>;

#define INSTANTIATE_STDLIB_ULTRA_TYPE(stdlib_type) template class stdlib_type<plonk::UltraComposer>;

#define INSTANTIATE_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                             \
    template class stdlib_type<plonk::UltraComposer, __VA_ARGS__>;
