#pragma once
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/hash/poseidon2/poseidon2.hpp"

namespace bb::crypto::merkle_tree {

typedef uint256_t index_t;

struct nullifier_leaf {
    fr value;
    index_t nextIndex;
    fr nextValue;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(value, nextIndex, nextValue);
    bool operator==(nullifier_leaf const&) const = default;

    std::ostream& operator<<(std::ostream& os)
    {
        os << "value = " << value << "\nnextIdx = " << nextIndex << "\nnextVal = " << nextValue;
        return os;
    }

    std::vector<fr> get_hash_inputs() const { return std::vector<fr>{ value, nextIndex, nextValue }; }

    static nullifier_leaf zero() { return nullifier_leaf{ .value = 0, .nextIndex = 0, .nextValue = 0 }; }
};

/**
 * @brief Wrapper for the Nullifier leaf class that allows for 0 values
 *
 */
template <typename HashingPolicy> class WrappedNullifierLeaf {

  public:
    // Initialize with a nullifier leaf
    WrappedNullifierLeaf(nullifier_leaf value)
        : data(value)
    {}
    // Initialize an empty leaf
    WrappedNullifierLeaf()
        : data(std::nullopt)
    {}

    bool operator==(WrappedNullifierLeaf const&) const = default;

    /**
     * @brief Pass through the underlying std::optional method
     *
     * @return true
     * @return false
     */
    bool has_value() const { return data.has_value(); }

    /**
     * @brief Return the wrapped nullifier_leaf object
     *
     * @return nullifier_leaf
     */
    nullifier_leaf unwrap() const { return data.value(); }

    /**
     * @brief Set the wrapped nullifier_leaf object value
     *
     * @param value
     */
    void set(nullifier_leaf value) { data.emplace(value); }

    /**
     * @brief Return the hash of the wrapped object, other return the zero hash of 0
     *
     * @return bb::fr
     */
    bb::fr hash() const
    {
        return data.has_value() ? HashingPolicy::hash(data.value().get_hash_inputs()) : bb::fr::zero();
    }

    /**
     * @brief Generate a zero leaf (call the constructor with no arguments)
     *
     * @return NullifierLeaf
     */
    static WrappedNullifierLeaf<HashingPolicy> zero() { return WrappedNullifierLeaf<HashingPolicy>(); }

  private:
    // Underlying data
    std::optional<nullifier_leaf> data;
};

template <typename HashingPolicy>
inline std::pair<size_t, bool> find_closest_leaf(std::vector<WrappedNullifierLeaf<HashingPolicy>> const& leaves_,
                                                 fr const& new_value)
{
    std::vector<uint256_t> diff;
    bool repeated = false;
    auto new_value_ = uint256_t(new_value);

    for (size_t i = 0; i < leaves_.size(); i++) {

        if (!leaves_[i].has_value()) {
            diff.push_back(new_value_);
            continue;
        }

        auto leaf_value_ = uint256_t(leaves_[i].unwrap().value);
        if (leaf_value_ > new_value_) {
            diff.push_back(leaf_value_);
        } else if (leaf_value_ == new_value_) {
            repeated = true;
            return std::make_pair(i, repeated);
        } else {
            diff.push_back(new_value_ - leaf_value_);
        }
    }
    auto it = std::min_element(diff.begin(), diff.end());
    return std::make_pair(static_cast<size_t>(it - diff.begin()), repeated);
}

} // namespace bb::crypto::merkle_tree
