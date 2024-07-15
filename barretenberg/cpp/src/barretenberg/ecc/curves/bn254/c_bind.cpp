#include "../bn254/fr.hpp"
#include "barretenberg/common/wasm_export.hpp"

using namespace bb;

WASM_EXPORT void bn254_fr_sqrt(uint8_t const* input, uint8_t* result)
{
    using serialize::write;
    auto input_fr = from_buffer<bb::fr>(input);
    auto [is_sqr, root] = input_fr.sqrt();

    uint8_t* is_sqrt_result_ptr = result;
    uint8_t* root_result_ptr = result + 1;

    write(is_sqrt_result_ptr, is_sqr);
    write(root_result_ptr, root);
}

// NOLINTEND(cert-dcl37-c, cert-dcl51-cpp, bugprone-reserved-identifier)