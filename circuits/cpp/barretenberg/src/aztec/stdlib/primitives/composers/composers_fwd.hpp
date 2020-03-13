#pragma once

namespace waffle {
class StandardComposer;
class MiMCComposer;
class TurboComposer;
} // namespace waffle

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<waffle::StandardComposer>;                                                       \
    extern template class stdlib_type<waffle::MiMCComposer>;                                                           \
    extern template class stdlib_type<waffle::TurboComposer>;

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<waffle::StandardComposer, __VA_ARGS__>;                                          \
    extern template class stdlib_type<waffle::MiMCComposer, __VA_ARGS__>;                                              \
    extern template class stdlib_type<waffle::TurboComposer, __VA_ARGS__>;
