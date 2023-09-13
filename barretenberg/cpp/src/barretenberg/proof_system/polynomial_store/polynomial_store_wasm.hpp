#pragma once
#include "barretenberg/polynomials/polynomial.hpp"
#include <string>
#include <unordered_map>

namespace proof_system {

template <typename Fr> class PolynomialStoreWasm {
  private:
    using Polynomial = barretenberg::Polynomial<Fr>;
    std::unordered_map<std::string, size_t> size_map;

  public:
    void put(std::string const& key, Polynomial&& value);

    Polynomial get(std::string const& key);
};

extern template class PolynomialStoreWasm<barretenberg::fr>;

} // namespace proof_system