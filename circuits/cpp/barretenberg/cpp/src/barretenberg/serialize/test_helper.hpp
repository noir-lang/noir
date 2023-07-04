#pragma once
#include "cbind.hpp"
#include <string>

/***
 * Do a roundtrip test encode/decode of an object.
 * @tparam T The object type.
 * @param object The object. Can be a default-initialized object.
 */
template <typename T> std::pair<T, T> msgpack_roundtrip(const T& object)
{
    T result;
    msgpack::sbuffer buffer;
    msgpack::pack(buffer, object);
    msgpack::unpack(buffer.data(), buffer.size()).get().convert(result);
    return { object, result };
}

template <typename T> inline T call_msgpack_cbind(auto cbind_func, auto... test_args)
{
    auto [input, input_len] = msgpack_encode_buffer(std::make_tuple(test_args...));
    uint8_t* output;
    size_t output_len;
    cbind_func(input, input_len, &output, &output_len);
    T actual_ret;
    msgpack::unpack((const char*)output, output_len).get().convert(actual_ret);
    aligned_free(output);
    return actual_ret;
}

// Running the end-to-end tests that msgpack bind creates
// This should suffice in testing the binding interface, function tests can be separate
inline auto call_func_and_wrapper(auto func, auto cbind_func, auto... test_args)
{
    auto expected_ret = func(test_args...);
    auto actual_ret = call_msgpack_cbind<decltype(expected_ret)>(cbind_func, test_args...);
    return std::make_pair(actual_ret, expected_ret);
}
