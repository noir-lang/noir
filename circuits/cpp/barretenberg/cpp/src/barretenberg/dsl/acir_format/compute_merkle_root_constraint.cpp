#include "compute_merkle_root_constraint.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"

namespace acir_format {

void create_compute_merkle_root_constraint(Composer& composer, const ComputeMerkleRootConstraint& constraint)
{
    // Convert value from a witness index into a field element.
    // This is the hash of the message. In Barretenberg, this would be input.value = hash_value(message)
    field_ct leaf = field_ct::from_witness_index(&composer, constraint.leaf);

    // Convert index from a witness index into a byte array
    field_ct index_field = field_ct::from_witness_index(&composer, constraint.index);
    auto index_bits = index_field.decompose_into_bits();

    // We are given the HashPath as a Vec<fr>
    // We want to first convert it into a Vec<(fr, fr)> then cast this to hash_path
    // struct which requires the method create_witness_hashpath
    hash_path_ct hash_path;

    // In Noir we accept a hash path that only contains one hash per tree level
    // It is ok to reuse the leaf as it will be overridden in check_subtree_membership when computing the current root
    // at each tree level
    for (size_t i = 0; i < constraint.hash_path.size(); i++) {
        if (!index_bits[i].get_value()) {
            field_ct left = leaf;
            field_ct right = field_ct::from_witness_index(&composer, constraint.hash_path[i]);
            hash_path.push_back(std::make_pair(left, right));
        } else {
            field_ct left = field_ct::from_witness_index(&composer, constraint.hash_path[i]);
            field_ct right = leaf;
            hash_path.push_back(std::make_pair(left, right));
        }
    }

    auto merkle_root = plonk::stdlib::merkle_tree::compute_subtree_root(hash_path, leaf, index_bits, 0);

    composer.assert_equal(merkle_root.witness_index, constraint.result);
}

} // namespace acir_format
