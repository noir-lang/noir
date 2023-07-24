#pragma once
// Note: heavy header due to serialization logic, don't include outside of tests
#include <barretenberg/serialize/msgpack_impl/msgpack_impl.hpp>

#include "barretenberg/common/throw_or_abort.hpp"
#include "schema_name.hpp"
#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <vector>

namespace msgpack {
template <typename T> uintptr_t __aligned_for(uintptr_t ptr)
{
    // Round to next alignment, (ptr % alignof(T)) == 0 after
    return ptr + (alignof(T) - (ptr % alignof(T))) % alignof(T);
}
template <typename T, typename... Args> std::string check_memory_span(T* obj, Args*... args)
{
    // We need to handle alignment. Thankfully, we have a tool here.
    // Convert the variadic template arguments to a vector of pairs.
    // Each pair contains a pointer (as uintptr_t) and its size.
    std::vector<std::pair<uintptr_t, size_t>> pointers{ { (uintptr_t)(args), sizeof(Args) }... };
    // Sort the vector based on the pointer values.
    std::sort(pointers.begin(), pointers.end(), [](const auto& a, const auto& b) { return a.first < b.first; });

    for (size_t i = 1; i < pointers.size(); ++i) {
        // Check if any of the Args* pointers overlap.
        auto last_end = pointers[i - 1].first + pointers[i - 1].second;
        if (last_end > pointers[i].first) {
            return "Overlap in " + msgpack_schema_name(*obj) + " MSGPACK_FIELDS() params detected!";
        }
        // Check if gap is too large.
        // Give some fuzzy room in case of 64 byte alignment restrictions.
        if (__aligned_for<T>(last_end) < pointers[i].first) {
            return "Gap in " + msgpack_schema_name(*obj) + " MSGPACK_FIELDS() params detected before member #" +
                   std::to_string(i) + " !";
        }
    }

    // Check if all Args* pointers exist in T* memory.
    uintptr_t t_start = reinterpret_cast<uintptr_t>(obj);
    uintptr_t t_end = t_start + sizeof(T);
    if (pointers.front().first < t_start || pointers.back().first + pointers.back().second > t_end) {
        return "Some " + msgpack_schema_name(*obj) + " MSGPACK_FIELDS() params don't exist in object!";
    }

    // Check if all of T* memory is used by the Args* pointers.
    size_t start = (size_t)obj;
    size_t end = (size_t)obj;
    for (auto [ptr, size] : pointers) {
        end = std::max(end, ptr + size);
    }
    size_t total_size = end - start;
    if (__aligned_for<T>(total_size) < sizeof(T)) {
        return "Incomplete " + msgpack_schema_name(*obj) + " MSGPACK_FIELDS() params! Not all of object specified.";
    }
    return {};
}

template <msgpack_concepts::HasMsgPack T> std::string check_msgpack_method(T& object)
{
    std::string result;
    auto checker = [&](auto&... values) { result = check_memory_span(&object, &values...); };
    object.msgpack([&](auto&... keys_and_values) { std::apply(checker, drop_keys(std::tie(keys_and_values...))); });
    return result;
}
void check_msgpack_usage(auto object)
{
    std::string result = check_msgpack_method(object);
    if (!result.empty()) {
        throw_or_abort(result);
    }
}
} // namespace msgpack