#include "polynomial_store_wasm.hpp"
#include "barretenberg/env/data_store.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

namespace proof_system {

template <typename Fr> void PolynomialStoreWasm<Fr>::put(std::string const& key, Polynomial&& value)
{
    // info("put ", key, ": ", value.hash());
    set_data(key.c_str(), (uint8_t*)value.data().get(), value.size() * sizeof(barretenberg::fr));
    size_map[key] = value.size();
};

template <typename Fr> barretenberg::Polynomial<Fr> PolynomialStoreWasm<Fr>::get(std::string const& key)
{
    auto p = Polynomial(size_map[key]);
    get_data(key.c_str(), (uint8_t*)p.data().get());
    // info("got (miss): ", key);
    return p;
};

template class PolynomialStoreWasm<barretenberg::fr>;

} // namespace proof_system