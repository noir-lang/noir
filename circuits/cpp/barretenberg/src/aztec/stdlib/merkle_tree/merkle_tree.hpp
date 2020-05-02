#pragma once
#include "hash_path.hpp"
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"
template <typename Composer>
bool_t<Composer> check_membership(Composer& composer,
                                  field_t<Composer> const& root,
                                  hash_path<Composer> const& hashes,
                                  byte_array<Composer> const& value,
                                  byte_array<Composer> const& index)
{
    field_t<Composer> current = hash_value(value);
    bool_t is_member = witness_t(&composer, true);

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
        current = pedersen<Composer>::compress(hashes[i].first, hashes[i].second);
    }

    // std::cout << "current " << current << " root " << root << std::endl;

    is_member &= current == root;
    return is_member;
}

template <typename Composer>
void assert_check_membership(Composer& composer,
                             field_t<Composer> const& root,
                             hash_path<Composer> const& hashes,
                             byte_array<Composer> const& value,
                             byte_array<Composer> const& index)
{
    auto exists = stdlib::merkle_tree::check_membership(composer, root, hashes, value, index);
    // std::cout << "assert check membership " << exists << std::endl;
    composer.assert_equal_constant(exists.witness_index, fr::one());
}

template <typename Composer>
void update_membership(Composer& composer,
                       field_t<Composer> const& new_root,
                       hash_path<Composer> const& new_hashes,
                       byte_array<Composer> const& new_value,
                       field_t<Composer> const& old_root,
                       hash_path<Composer> const& old_hashes,
                       byte_array<Composer> const& old_value,
                       byte_array<Composer> const& index)
{
    // Check old path hashes lead to the old root. They're used when validating the new path hashes.
    assert_check_membership(composer, old_root, old_hashes, old_value, index);

    // Check the new path hashes lead from the new value to the new root.
    assert_check_membership(composer, new_root, new_hashes, new_value, index);

    // Check that only the appropriate left or right hash was updated in the new hash path.
    for (size_t i = 0; i < new_hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);
        bool_t share_left = (old_hashes[i].first == new_hashes[i].first) & path_bit;
        bool_t share_right = (old_hashes[i].second == new_hashes[i].second) & !path_bit;
        composer.assert_equal_constant((share_left ^ share_right).witness_index, barretenberg::fr::one());
    }
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk