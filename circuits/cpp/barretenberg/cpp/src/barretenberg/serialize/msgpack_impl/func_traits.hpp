#pragma once
#include "../msgpack.hpp"
#include <tuple>

// Define a template struct to deduce function traits for different function types
template <typename Func> struct func_traits;

// Specialization for function pointers
template <typename R, typename... Vs> struct func_traits<R (*)(Vs...)> {
    typedef std::tuple<typename std::decay<Vs>::type...> Args; // Define a tuple type that holds all argument types
    Args args;                                                 // Args instance
    R ret;                                                     // Holds return type
    MSGPACK_FIELDS(args, ret); // Macro from msgpack library to serialize/deserialize fields
};

// Specialization for function references
template <typename R, typename... Vs> struct func_traits<R (&)(Vs...)> {
    typedef std::tuple<typename std::decay<Vs>::type...> Args;
    Args args;
    R ret;
    MSGPACK_FIELDS(args, ret);
};

// Specialization for member function pointers. This also includes lambda types,
// as they are functors (objects with operator()) and hence have a member function pointer
template <typename R, typename T, typename... Vs> struct func_traits<R (T::*)(Vs...) const> {
    typedef std::tuple<typename std::decay<Vs>::type...> Args;
    Args args;
    R ret;
    MSGPACK_FIELDS(args, ret);
};

// Define a concept that checks if the type is a lambda (or functor) type
// This is done by checking if T::operator() exists
template <typename T> concept LambdaType = requires()
{
    typename std::enable_if_t<std::is_member_function_pointer_v<decltype(&T::operator())>, void>;
};

// Overload for lambda (or functor) types
template <LambdaType T> constexpr auto get_func_traits()
{
    // If T is a lambda type (i.e. it has operator()), deduce its traits using func_traits
    return func_traits<decltype(&T::operator())>();
}

// Overload for non-lambda types
template <typename T> constexpr auto get_func_traits()
{
    // If T is not a lambda, just deduce its traits using func_traits
    return func_traits<T>();
}
