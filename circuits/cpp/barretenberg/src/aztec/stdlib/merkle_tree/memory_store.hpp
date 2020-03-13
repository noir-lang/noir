#pragma once
#include "hash_path.hpp"

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

class MemoryStore {
  public:
    MemoryStore(size_t depth);

    fr_hash_path get_hash_path(size_t index);

    void update_element(size_t index, std::string const& value);

    std::string const& get_element(size_t index);

    fr root() const { return root_; }

  private:
    size_t depth_;
    size_t total_size_;
    barretenberg::fr root_;
    std::vector<barretenberg::fr> hashes_;
    std::vector<std::string> preimages_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk