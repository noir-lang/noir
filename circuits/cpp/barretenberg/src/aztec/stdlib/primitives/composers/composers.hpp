#pragma once
#include <plonk/composer/mimc_composer.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/plookup_composer.hpp>

#define INSTANTIATE_STDLIB_TYPE(stdlib_type)                                                                           \
    template class stdlib_type<waffle::StandardComposer>;                                                              \
    template class stdlib_type<waffle::MiMCComposer>;                                                                  \
    template class stdlib_type<waffle::TurboComposer>;                                                                 \
    template class stdlib_type<waffle::PLookupComposer>;

#define INSTANTIATE_STDLIB_TYPE_VA(stdlib_type, ...)                                                                   \
    template class stdlib_type<waffle::StandardComposer, __VA_ARGS__>;                                                 \
    template class stdlib_type<waffle::MiMCComposer, __VA_ARGS__>;                                                     \
    template class stdlib_type<waffle::TurboComposer, __VA_ARGS__>;                                                    \
    template class stdlib_type<waffle::PLookupComposer, __VA_ARGS__>;
