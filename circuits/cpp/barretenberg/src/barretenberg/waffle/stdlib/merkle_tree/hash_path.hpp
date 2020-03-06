#pragma once
#include "../field/field.hpp"
#include "../int_utils.hpp"
#include "hash.hpp"
#include <algorithm>
#include <vector>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;
using namespace int_utils;

typedef std::vector<std::pair<fr, fr>> fr_hash_path;
template <typename Ctx> using hash_path = std::vector<std::pair<field_t<Ctx>, field_t<Ctx>>>;

inline fr_hash_path get_new_hash_path(fr_hash_path const& old_path, uint128_t index, std::string const& value)
{
    fr_hash_path path = old_path;
    fr current = hash_value_native(value);
    for (size_t i = 0; i < old_path.size(); ++i) {
        bool path_bit = index & 0x1;
        if (path_bit) {
            path[i].second = current;
        } else {
            path[i].first = current;
        }
        current = compress_native({ path[i].first, path[i].second });
        index /= 2;
    }
    return path;
}

template <typename Ctx> inline hash_path<Ctx> create_witness_hash_path(Ctx& ctx, fr_hash_path const& input)
{
    hash_path<Ctx> result;
    std::transform(input.begin(), input.end(), std::back_inserter(result), [&](auto const& v) {
        return std::make_pair(field_t(witness_t(&ctx, v.first)), field_t(witness_t(&ctx, v.second)));
    });
    return result;
}

inline fr get_hash_path_root(fr_hash_path const& input)
{
    return compress_native({ input[input.size() - 1].first, input[input.size() - 1].second });
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk

// We add to std namespace as fr_hash_path is actually a std::vector, and this is the only way
// to achieve effective ADL.
namespace std {
template <typename Ctx>
inline std::ostream& operator<<(std::ostream& os, plonk::stdlib::merkle_tree::hash_path<Ctx> const& path)
{
    os << "[\n";
    for (size_t i = 0; i < path.size(); ++i) {
        os << "  (" << i << ": " << path[i].first << ", " << path[i].second << ")\n";
    }
    os << "]";
    return os;
}
inline std::ostream& operator<<(std::ostream& os, plonk::stdlib::merkle_tree::fr_hash_path const& path)
{
    os << "[\n";
    for (size_t i = 0; i < path.size(); ++i) {
        os << "  (" << i << ": " << path[i].first << ", " << path[i].second << ")\n";
    }
    os << "]";
    return os;
}
} // namespace std