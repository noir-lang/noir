#include "polynomial_store_wasm.hpp"
#include "barretenberg/env/data_store.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

namespace bb {

template <typename Fr> void PolynomialStoreWasm<Fr>::put(std::string const& key, Polynomial&& value)
{
    // info("put ", key, ": ", value.hash());
    set_data(key.c_str(), (uint8_t*)value.data().get(), value.size() * sizeof(bb::fr));
    size_map[key] = value.size();
};

template <typename Fr> bb::Polynomial<Fr> PolynomialStoreWasm<Fr>::get(std::string const& key)
{
    auto p = Polynomial(size_map[key]);
    get_data(key.c_str(), (uint8_t*)p.data().get());
    // info("got (miss): ", key);
    return p;
};

template class PolynomialStoreWasm<bb::fr>;

} // namespace bb