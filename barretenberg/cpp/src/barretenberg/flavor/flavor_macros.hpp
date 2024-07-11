#pragma once

// Macros for defining the flavor classes.
// These are used to derive iterator methods along with the body of a 'flavor' class.
// DEFINE_FLAVOR_MEMBERS lets you define a flavor entity as a collection of individual members, and derive an iterator.
// while DEFINE_COMPOUND_GET_ALL lets you combine the iterators of substructures or base
// classes.

#include "barretenberg/common/ref_array.hpp"
#include "barretenberg/common/std_array.hpp"
#include "barretenberg/common/std_string.hpp"
#include "barretenberg/common/std_vector.hpp"
#include <array>
#include <iostream>
#include <sstream>

namespace bb::detail {
template <typename... Args> constexpr std::size_t _va_count(Args&&... /*unused*/)
{
    return sizeof...(Args);
}
template <typename T, typename... BaseClass> constexpr std::size_t _sum_base_class_size(const T& arg)
{
    return (static_cast<const BaseClass&>(arg).size() + ...);
}
template <typename T, typename... BaseClass> auto _concatenate_base_class_get_all(T& arg)
{
    return concatenate(static_cast<BaseClass&>(arg).get_all()...);
}
template <typename T, typename... BaseClass> auto _concatenate_base_class_get_all_const(const T& arg)
{
    return concatenate(static_cast<const BaseClass&>(arg).get_all()...);
}
template <typename T, typename... BaseClass> auto _concatenate_base_class_get_labels(const T& arg)
{
    return concatenate(static_cast<const BaseClass&>(arg).get_labels()...);
}
} // namespace bb::detail

// Needed to force expansion of __VA_ARGS__ before converting to string.
#define VARARGS_TO_STRING(...) #__VA_ARGS__

#define DEFINE_REF_VIEW(...)                                                                                           \
    [[nodiscard]] auto get_all()                                                                                       \
    {                                                                                                                  \
        return RefArray{ __VA_ARGS__ };                                                                                \
    }                                                                                                                  \
    [[nodiscard]] auto get_all() const                                                                                 \
    {                                                                                                                  \
        return RefArray{ __VA_ARGS__ };                                                                                \
    }

/**
 * @brief Define the body of a flavor class, included each member and a pointer view with which to iterate the struct.
 *
 * @tparam T The underlying data type stored in the array
 * @tparam HandleType The type that will be used to
 * @tparam NUM_ENTITIES The size of the underlying array.
 */
#define DEFINE_FLAVOR_MEMBERS(DataType, ...)                                                                           \
    DataType __VA_ARGS__;                                                                                              \
    DEFINE_REF_VIEW(__VA_ARGS__)                                                                                       \
    const std::vector<std::string>& get_labels() const                                                                 \
    {                                                                                                                  \
        static const std::vector<std::string> labels =                                                                 \
            bb::detail::split_and_trim(VARARGS_TO_STRING(__VA_ARGS__), ',');                                           \
        return labels;                                                                                                 \
    }                                                                                                                  \
    constexpr std::size_t size() const                                                                                 \
    {                                                                                                                  \
        return bb::detail::_va_count(__VA_ARGS__);                                                                     \
    }

#define DEFINE_COMPOUND_GET_ALL(...)                                                                                   \
    [[nodiscard]] auto get_all()                                                                                       \
    {                                                                                                                  \
        return bb::detail::_concatenate_base_class_get_all<decltype(*this), __VA_ARGS__>(*this);                       \
    }                                                                                                                  \
    [[nodiscard]] auto get_all() const                                                                                 \
    {                                                                                                                  \
        return bb::detail::_concatenate_base_class_get_all_const<decltype(*this), __VA_ARGS__>(*this);                 \
    }                                                                                                                  \
    constexpr std::size_t size() const                                                                                 \
    {                                                                                                                  \
        return bb::detail::_sum_base_class_size<decltype(*this), __VA_ARGS__>(*this);                                  \
    }                                                                                                                  \
    std::vector<std::string> get_labels() const                                                                        \
    {                                                                                                                  \
        return bb::detail::_concatenate_base_class_get_labels<decltype(*this), __VA_ARGS__>(*this);                    \
    }
