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

    fr update_element(size_t index, fr const& value);

    fr root() const { return root_; }

  private:
    size_t depth_;
    size_t total_size_;
    barretenberg::fr root_;
    std::vector<barretenberg::fr> hashes_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk