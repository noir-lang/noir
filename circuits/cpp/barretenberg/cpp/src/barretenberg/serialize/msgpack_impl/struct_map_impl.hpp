#pragma once
// Note: heavy header due to serialization logic, don't include if msgpack.hpp will do
#include <cassert>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#define MSGPACK_NO_BOOST
#include "concepts.hpp"
#include "drop_keys.hpp"
#include <msgpack.hpp>

namespace msgpack::adaptor {
// reads structs with msgpack() method from a JSON-like dictionary
template <msgpack_concepts::HasMsgPack T> struct convert<T> {
    msgpack::object const& operator()(msgpack::object const& o, T& v) const
    {
        static_assert(std::is_default_constructible_v<T>,
                      "MSGPACK_FIELDS requires default-constructible types (used during unpacking)");
        v.msgpack([&](auto&... args) {
            auto static_checker = [&](auto&... value_args) {
                static_assert(msgpack_concepts::MsgpackConstructible<T, decltype(value_args)...>,
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
                static_assert(msgpack_concepts::MsgpackConstructible<T, decltype(value_args)...>,
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
