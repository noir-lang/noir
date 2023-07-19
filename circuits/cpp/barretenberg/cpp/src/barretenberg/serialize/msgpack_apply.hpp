#pragma once

#include "msgpack.hpp"
#include "msgpack_impl/drop_keys.hpp"

namespace msgpack {
/**
 * @brief Helper method for better error reporting. Clang does not give the best errors for lambdas.
 */
template <msgpack_concepts::HasMsgPack T> void msgpack_apply(const auto& func, auto&... args)
{
    std::apply(func, msgpack::drop_keys(std::tie(args...)));
}
/**
 * @brief Applies a function to the values exposed by the msgpack method.
 * @param value The value whose fields to reflect over.
 * @param func The function to call with each field as an argument.
 */
template <msgpack_concepts::HasMsgPack T> void msgpack_apply(const T& value, const auto& func)
{
    auto static_checker = [&](auto&... value_args) {
        static_assert(msgpack_concepts::MsgpackConstructible<T, decltype(value_args)...>,
                      "MSGPACK_FIELDS requires a constructor that can take the types listed in MSGPACK_FIELDS. "
                      "Type or arg count mismatch, or member initializer constructor not available.");
    };
    // We must use const_cast as our method is meant to be polymorphic over const, but there's no such concept in C++
    const_cast<T&>(value).msgpack([&](auto&... args) { // NOLINT
        std::apply(static_checker, msgpack::drop_keys(std::tie(args...)));
        msgpack_apply<T>(func, args...);
    });
}
} // namespace msgpack
