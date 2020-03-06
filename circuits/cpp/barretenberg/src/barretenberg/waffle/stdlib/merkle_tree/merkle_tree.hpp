#pragma once
#include "../bitarray/bitarray.hpp"
#include "../byte_array/byte_array.hpp"
#include "../crypto/hash/pedersen.hpp"
#include "../field/field.hpp"
#include "../int_utils.hpp"
#include "leveldb_store.hpp"
#include "memory_store.hpp"

namespace waffle {
class TurboComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace int_utils;

template <typename Ctx>
bool_t<Ctx> check_membership(Ctx& ctx,
                             field_t<Ctx> const& root,
                             hash_path<Ctx> const& hashes,
                             byte_array<Ctx> const& value,
                             byte_array<Ctx> const& index)
{
    field_t current = hash_value(value);
    bool_t is_member = witness_t(&ctx, true);

    for (size_t i = 0; i < hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);

        bool_t is_left = (current == hashes[i].first) & !path_bit;
        bool_t is_right = (current == hashes[i].second) & path_bit;
        is_member &= is_left ^ is_right;
        // if (!is_member.get_value()) {
        //     std::cout << "failed at height " << i << std::endl;
        //     std::cout << "is_left " << is_left.get_value() << std::endl;
        //     std::cout << "is_right " << is_right.get_value() << std::endl;
        // }
        current = pedersen::compress(hashes[i].first, hashes[i].second);
    }

    // std::cout << "current " << current << " root " << root << std::endl;

    is_member &= current == root;
    return is_member;
}

template <typename Ctx>
void assert_check_membership(Ctx& ctx,
                             field_t<Ctx> const& root,
                             hash_path<Ctx> const& hashes,
                             byte_array<Ctx> const& value,
                             byte_array<Ctx> const& index)
{
    auto exists = stdlib::merkle_tree::check_membership(ctx, root, hashes, value, index);
    // std::cout << "assert check membership " << exists << std::endl;
    ctx.assert_equal_constant(exists.witness_index, fr::one());
}

template <typename Ctx>
void update_membership(Ctx& ctx,
                       field_t<Ctx> const& new_root,
                       hash_path<Ctx> const& new_hashes,
                       byte_array<Ctx> const& new_value,
                       field_t<Ctx> const& old_root,
                       hash_path<Ctx> const& old_hashes,
                       byte_array<Ctx> const& old_value,
                       byte_array<Ctx> const& index)
{
    // Check old path hashes lead to the old root. They're used when validating the new path hashes.
    assert_check_membership(ctx, old_root, old_hashes, old_value, index);

    // Check the new path hashes lead from the new value to the new root.
    assert_check_membership(ctx, new_root, new_hashes, new_value, index);

    // Check that only the appropriate left or right hash was updated in the new hash path.
    for (size_t i = 0; i < new_hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);
        bool_t share_left = (old_hashes[i].first == new_hashes[i].first) & path_bit;
        bool_t share_right = (old_hashes[i].second == new_hashes[i].second) & !path_bit;
        ctx.assert_equal_constant((share_left ^ share_right).witness_index, barretenberg::fr::one());
    }
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk