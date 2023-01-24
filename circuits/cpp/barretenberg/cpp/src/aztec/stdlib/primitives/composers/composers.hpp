#pragma once
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/plookup_composer.hpp>

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<waffle::StandardComposer>;                                                              \
    template class stdlib_type<waffle::TurboComposer>;                                                                 \
    template class stdlib_type<waffle::PlookupComposer>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<waffle::StandardComposer, __VA_ARGS__>;                                                 \
    template class stdlib_type<waffle::TurboComposer, __VA_ARGS__>;                                                    \
    template class stdlib_type<waffle::PlookupComposer, __VA_ARGS__>;
