#include "c_bind.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "simple/simple.hpp"

using namespace bb::plonk::stdlib::types;

WASM_EXPORT void examples_simple_create_and_verify_proof(bool* valid)
{
    auto ptrs = examples::simple::create_builder_and_composer();
    auto proof = examples::simple::create_proof(ptrs);
    *valid = examples::simple::verify_proof(ptrs, proof);
    examples::simple::delete_builder_and_composer(ptrs);
}
