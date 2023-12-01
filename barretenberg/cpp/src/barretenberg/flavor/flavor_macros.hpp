#pragma once

// Macros for defining the flavor classes.
// These are used to derive iterator methods along with the body of a 'flavor' class.
// DEFINE_FLAVOR_MEMBERS lets you define a flavor entity as a collection of individual members, and derive an iterator.
// while DEFINE_COMPOUND_GET_ALL lets you combine the iterators of substructures or base
// classes.

#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/common/std_array.hpp"
#include <array>
#include <iostream>
#include <sstream>

template <typename... Refs> auto _refs_to_pointer_array(Refs&... refs)
{
    return std::array{ &refs... };
}

#define DEFINE_REF_VIEW(...)                                                                                           \
    [[nodiscard]] auto get_all()                                                                                       \
    {                                                                                                                  \
        return RefVector{ __VA_ARGS__ };                                                                               \
    }                                                                                                                  \
    [[nodiscard]] auto get_all() const                                                                                 \
    {                                                                                                                  \
        return RefVector{ __VA_ARGS__ };                                                                               \
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
    DEFINE_REF_VIEW(__VA_ARGS__)

#define DEFINE_COMPOUND_GET_ALL(...)                                                                                   \
    [[nodiscard]] auto get_all()                                                                                       \
    {                                                                                                                  \
        return concatenate(__VA_ARGS__);                                                                               \
    }                                                                                                                  \
    [[nodiscard]] auto get_all() const                                                                                 \
    {                                                                                                                  \
        return concatenate(__VA_ARGS__);                                                                               \
    }
