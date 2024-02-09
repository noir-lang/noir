#include "leaves_cache.hpp"

namespace bb::crypto::merkle_tree {

index_t LeavesCache::get_size() const
{
    return index_t(leaves_.size());
}

std::pair<bool, index_t> LeavesCache::find_low_value(const fr& new_value) const
{
    std::map<uint256_t, index_t>::const_iterator it = indices_.lower_bound(new_value);
    if (it == indices_.end()) {
        // there is no element >= the requested value.
        // decrement the iterator to get the value preceeding the requested value
        --it;
        return std::make_pair(false, it->second);
    }
    if (it->first == uint256_t(new_value)) {
        // the value is already present and the iterator points to it
        return std::make_pair(true, it->second);
    }
    // the iterator points to the element immediately larger than the requested value
    --it;
    //  it now points to the value less than that requested
    return std::make_pair(false, it->second);
}
indexed_leaf LeavesCache::get_leaf(const index_t& index) const
{
    ASSERT(index >= 0 && index < leaves_.size());
    return leaves_[size_t(index)];
}
void LeavesCache::set_at_index(const index_t& index, const indexed_leaf& leaf, bool add_to_index)
{
    if (index >= leaves_.size()) {
        leaves_.resize(size_t(index + 1));
    }
    leaves_[size_t(index)] = leaf;
    if (add_to_index) {
        indices_[uint256_t(leaf.value)] = index;
    }
}
void LeavesCache::append_leaf(const indexed_leaf& leaf)
{
    index_t next_index = leaves_.size();
    set_at_index(next_index, leaf, true);
}

} // namespace bb::crypto::merkle_tree