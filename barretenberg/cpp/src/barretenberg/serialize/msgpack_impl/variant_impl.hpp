#pragma once
// Note: Meant to only be included in compilation units that need msgpack
#define MSGPACK_NO_BOOST
#include <msgpack.hpp>
#include <variant>

namespace msgpack::adaptor {
// writes std::variant to msgpack format (TODO should we read std::variant?)
template <typename... T> struct pack<std::variant<T...>> {
    auto& operator()(auto& o, std::variant<T...> const& variant) const
    {
        std::visit([&o](const auto& arg) { o.pack(arg); }, variant);
        return o;
    }
};
} // namespace msgpack::adaptor
