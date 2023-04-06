#pragma once
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;
typedef uint256_t index_t;

struct nullifier_leaf {
    fr value;
    index_t nextIndex;
    fr nextValue;

    bool operator==(nullifier_leaf const&) const = default;

    std::ostream& operator<<(std::ostream& os)
    {
        os << "value = " << value << "\nnextIdx = " << nextIndex << "\nnextVal = " << nextValue;
        return os;
    }

    void read(uint8_t const*& it)
    {
        using serialize::read;
        read(it, value);
        read(it, nextIndex);
        read(it, nextValue);
    }

    inline void write(std::vector<uint8_t>& buf)
    {
        using serialize::write;
        write(buf, value);
        write(buf, nextIndex);
        write(buf, nextValue);
    }

    barretenberg::fr hash() const { return stdlib::merkle_tree::hash_multiple_native({ value, nextIndex, nextValue }); }
};

inline std::pair<size_t, bool> find_closest_leaf(std::vector<nullifier_leaf> const& leaves_, fr const& new_value)
{
    std::vector<uint256_t> diff;
    bool repeated = false;
    auto new_value_ = uint256_t(new_value);

    for (size_t i = 0; i < leaves_.size(); i++) {
        auto leaf_value_ = uint256_t(leaves_[i].value);
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

} // namespace merkle_tree
} // namespace stdlib
} // namespace proof_system::plonk