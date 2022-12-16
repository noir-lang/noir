#pragma once

namespace waffle {
class StandardComposer;
class TurboComposer;
class UltraComposer;
} // namespace waffle

#define EXTERN_STDLIB_TYPE(stdlib_type)                                                                                \
    extern template class stdlib_type<waffle::StandardComposer>;                                                       \
    extern template class stdlib_type<waffle::TurboComposer>;                                                          \
    extern template class stdlib_type<waffle::UltraComposer>;

#define EXTERN_STDLIB_TYPE_VA(stdlib_type, ...)                                                                        \
    extern template class stdlib_type<waffle::StandardComposer, __VA_ARGS__>;                                          \
    extern template class stdlib_type<waffle::TurboComposer, __VA_ARGS__>;                                             \
    extern template class stdlib_type<waffle::UltraComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_BASIC_TYPE(stdlib_type)                                                                          \
    extern template class stdlib_type<waffle::StandardComposer>;                                                       \
    extern template class stdlib_type<waffle::TurboComposer>;

#define EXTERN_STDLIB_BASIC_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<waffle::StandardComposer, __VA_ARGS__>;                                          \
    extern template class stdlib_type<waffle::TurboComposer, __VA_ARGS__>;

#define EXTERN_STDLIB_ULTRA_TYPE(stdlib_type) extern template class stdlib_type<waffle::UltraComposer>;

#define EXTERN_STDLIB_ULTRA_TYPE_VA(stdlib_type, ...)                                                                  \
    extern template class stdlib_type<waffle::UltraComposer, __VA_ARGS__>;
