#pragma once
// Note: heavy header due to serialization logic, don't include if msgpack.hpp will do
#include <string>
#include <iostream>
#include <iomanip>
#include <sstream>
#include <cassert>
#define MSGPACK_NO_BOOST
#include <msgpack.hpp>
#include "concepts.hpp"

namespace msgpack {
template <typename Tuple, std::size_t... Is> auto drop_keys_impl(Tuple&& tuple, std::index_sequence<Is...>)
{
    // Expand 0 to n/2 to 1 to n+1 (increments of 2)
    // Use it to filter the tuple
    return std::tie(std::get<Is * 2 + 1>(std::forward<Tuple>(tuple))...);
}

template <typename... Args> auto drop_keys(std::tuple<Args...>&& tuple)
{
    static_assert(sizeof...(Args) % 2 == 0, "Tuple must contain an even number of elements");
    // Compile time sequence of integers from 0 to n/2
    auto compile_time_0_to_n_div_2 = std::make_index_sequence<sizeof...(Args) / 2>{};
    return drop_keys_impl(tuple, compile_time_0_to_n_div_2);
}
} // namespace msgpack

namespace msgpack {
template <typename T, typename... Args> concept MsgpackConstructible = requires(T object, Args... args)
{
    T{ args... };
};
} // namespace msgpack

namespace msgpack::adaptor {
// reads structs with msgpack() method from a JSON-like dictionary
template <msgpack_concepts::HasMsgPack T> struct convert<T> {
    msgpack::object const& operator()(msgpack::object const& o, T& v) const
    {
        static_assert(std::is_default_constructible_v<T>,
                      "MSGPACK_FIELDS requires default-constructible types (used during unpacking)");
        v.msgpack([&](auto&... args) {
            auto static_checker = [&](auto&... value_args) {
                static_assert(msgpack::MsgpackConstructible<T, decltype(value_args)...>,
                              "MSGPACK_FIELDS requires a constructor that can take the types listed in MSGPACK_FIELDS. "
                              "Type or arg count mismatch, or member initializer constructor not available.");
            };
            std::apply(static_checker, drop_keys(std::tie(args...)));
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_unpack(o);
        });
        return o;
    }
};

// converts structs with msgpack() method from a JSON-like dictionary
template <msgpack_concepts::HasMsgPack T> struct pack<T> {
    template <typename Stream> packer<Stream>& operator()(msgpack::packer<Stream>& o, T const& v) const
    {
        static_assert(std::is_default_constructible_v<T>,
                      "MSGPACK_FIELDS requires default-constructible types (used during unpacking)");
        const_cast<T&>(v).msgpack([&](auto&... args) {
            auto static_checker = [&](auto&... value_args) {
                static_assert(msgpack::MsgpackConstructible<T, decltype(value_args)...>,
                              "T requires a constructor that can take the fields listed in MSGPACK_FIELDS (T will be "
                              "in template parameters in the compiler stack trace)"
                              "Check the MSGPACK_FIELDS macro usage in T for incompleteness or wrong order."
                              "Alternatively, a matching member initializer constructor might not be available for T "
                              "and should be defined.");
            };
            std::apply(static_checker, drop_keys(std::tie(args...)));
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(o);
        });
        return o;
    }
};

} // namespace msgpack::adaptor
