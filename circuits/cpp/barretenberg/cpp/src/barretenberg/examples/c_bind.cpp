#include "c_bind.hpp"
#include "simple/simple.hpp"
#include "barretenberg/srs/global_crs.hpp"

using namespace proof_system::plonk::stdlib::types;

WASM_EXPORT void examples_simple_create_and_verify_proof(bool* valid)
{
    auto* composer_ptr = examples::simple::create_composer(barretenberg::srs::get_crs_factory());
    auto proof = examples::simple::create_proof(composer_ptr);
    *valid = examples::simple::verify_proof(composer_ptr, proof);
    examples::simple::delete_composer(composer_ptr);
}
