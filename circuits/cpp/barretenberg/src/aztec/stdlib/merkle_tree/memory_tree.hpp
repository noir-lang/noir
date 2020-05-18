#pragma once
#include "hash_path.hpp"

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

class MemoryTree {
  public:
    MemoryTree(size_t depth);

    fr_hash_path get_hash_path(size_t index);

    template <size_t S> fr update_element(size_t index, std::array<uint8_t, S> const& value)
    {
        return update_element(index, std::vector(value.begin(), value.end()));
    }

    fr update_element(size_t index, std::vector<uint8_t> const& value);

    std::vector<uint8_t> const& get_element(size_t index);

    fr root() const { return root_; }

  private:
    size_t depth_;
    size_t total_size_;
    barretenberg::fr root_;
    std::vector<barretenberg::fr> hashes_;
    std::vector<std::vector<uint8_t>> preimages_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk