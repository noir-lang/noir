#include "c_bind.hpp"
#include "../acir_format/acir_to_constraint_buf.hpp"
#include "acir_composer.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/net.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/dsl/acir_format/acir_format.hpp"
#include "barretenberg/dsl/acir_proofs/goblin_acir_composer.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include <cstdint>
#include <memory>

WASM_EXPORT void acir_get_circuit_sizes(uint8_t const* acir_vec, uint32_t* exact, uint32_t* total, uint32_t* subgroup)
{
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    auto builder = acir_format::create_circuit(constraint_system, 1 << 19);
    *exact = htonl((uint32_t)builder.get_num_gates());
    *total = htonl((uint32_t)builder.get_total_circuit_size());
    *subgroup = htonl((uint32_t)builder.get_circuit_subgroup_size(builder.get_total_circuit_size()));
}

WASM_EXPORT void acir_new_acir_composer(uint32_t const* size_hint, out_ptr out)
{
    *out = new acir_proofs::AcirComposer(ntohl(*size_hint));
}

WASM_EXPORT void acir_new_goblin_acir_composer(out_ptr out)
{
    *out = new acir_proofs::GoblinAcirComposer();
}

WASM_EXPORT void acir_delete_acir_composer(in_ptr acir_composer_ptr)
{
    delete reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
}

WASM_EXPORT void acir_init_proving_key(in_ptr acir_composer_ptr, uint8_t const* acir_vec)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    acir_composer->create_circuit(constraint_system);

    acir_composer->init_proving_key();
}

WASM_EXPORT void acir_create_proof(in_ptr acir_composer_ptr,
                                   uint8_t const* acir_vec,
                                   uint8_t const* witness_vec,
                                   bool const* is_recursive,
                                   uint8_t** out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    auto witness = acir_format::witness_buf_to_witness_data(from_buffer<std::vector<uint8_t>>(witness_vec));

    acir_composer->create_circuit(constraint_system, witness);

    acir_composer->init_proving_key();
    auto proof_data = acir_composer->create_proof(*is_recursive);
    *out = to_heap_buffer(proof_data);
}

WASM_EXPORT void acir_goblin_accumulate(in_ptr acir_composer_ptr,
                                        uint8_t const* acir_vec,
                                        uint8_t const* witness_vec,
                                        uint8_t** out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::GoblinAcirComposer*>(*acir_composer_ptr);
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    auto witness = acir_format::witness_buf_to_witness_data(from_buffer<std::vector<uint8_t>>(witness_vec));

    acir_composer->create_circuit(constraint_system, witness);
    auto proof_data = acir_composer->accumulate();
    *out = to_heap_buffer(proof_data);
}

WASM_EXPORT void acir_goblin_prove(in_ptr acir_composer_ptr,
                                   uint8_t const* acir_vec,
                                   uint8_t const* witness_vec,
                                   uint8_t** out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::GoblinAcirComposer*>(*acir_composer_ptr);
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    auto witness = acir_format::witness_buf_to_witness_data(from_buffer<std::vector<uint8_t>>(witness_vec));

    acir_composer->create_circuit(constraint_system, witness);
    auto proof_data = acir_composer->accumulate_and_prove();
    *out = to_heap_buffer(proof_data);
}

WASM_EXPORT void acir_load_verification_key(in_ptr acir_composer_ptr, uint8_t const* vk_buf)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto vk_data = from_buffer<plonk::verification_key_data>(vk_buf);
    acir_composer->load_verification_key(std::move(vk_data));
}

WASM_EXPORT void acir_init_verification_key(in_ptr acir_composer_ptr)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    acir_composer->init_verification_key();
}

WASM_EXPORT void acir_get_verification_key(in_ptr acir_composer_ptr, uint8_t** out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto vk = acir_composer->init_verification_key();
    // We flatten to a vector<uint8_t> first, as that's how we treat it on the calling side.
    *out = to_heap_buffer(to_buffer(*vk));
}

WASM_EXPORT void acir_get_proving_key(in_ptr acir_composer_ptr, uint8_t const* acir_vec, uint8_t** out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto constraint_system = acir_format::circuit_buf_to_acir_format(from_buffer<std::vector<uint8_t>>(acir_vec));
    acir_composer->create_circuit(constraint_system);
    auto pk = acir_composer->init_proving_key();
    // We flatten to a vector<uint8_t> first, as that's how we treat it on the calling side.
    *out = to_heap_buffer(to_buffer(*pk));
}

WASM_EXPORT void acir_goblin_verify_accumulator(in_ptr acir_composer_ptr, uint8_t const* proof_buf, bool* result)
{
    auto acir_composer = reinterpret_cast<acir_proofs::GoblinAcirComposer*>(*acir_composer_ptr);
    auto proof = from_buffer<std::vector<uint8_t>>(proof_buf);
    *result = acir_composer->verify_accumulator(proof);
}

WASM_EXPORT void acir_goblin_verify(in_ptr acir_composer_ptr, uint8_t const* proof_buf, bool* result)
{
    auto acir_composer = reinterpret_cast<acir_proofs::GoblinAcirComposer*>(*acir_composer_ptr);
    auto proof = from_buffer<std::vector<uint8_t>>(proof_buf);
    *result = acir_composer->verify(proof);
}

WASM_EXPORT void acir_verify_proof(in_ptr acir_composer_ptr,
                                   uint8_t const* proof_buf,
                                   bool const* is_recursive,
                                   bool* result)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto proof = from_buffer<std::vector<uint8_t>>(proof_buf);
    *result = acir_composer->verify_proof(proof, *is_recursive);
}

WASM_EXPORT void acir_get_solidity_verifier(in_ptr acir_composer_ptr, out_str_buf out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto str = acir_composer->get_solidity_verifier();
    *out = to_heap_buffer(str);
}

WASM_EXPORT void acir_serialize_proof_into_fields(in_ptr acir_composer_ptr,
                                                  uint8_t const* proof_buf,
                                                  uint32_t const* num_inner_public_inputs,
                                                  fr::vec_out_buf out)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);
    auto proof = from_buffer<std::vector<uint8_t>>(proof_buf);
    auto proof_as_fields = acir_composer->serialize_proof_into_fields(proof, ntohl(*num_inner_public_inputs));

    *out = to_heap_buffer(proof_as_fields);
}

WASM_EXPORT void acir_serialize_verification_key_into_fields(in_ptr acir_composer_ptr,
                                                             fr::vec_out_buf out_vkey,
                                                             fr::out_buf out_key_hash)
{
    auto acir_composer = reinterpret_cast<acir_proofs::AcirComposer*>(*acir_composer_ptr);

    auto vkey_as_fields = acir_composer->serialize_verification_key_into_fields();
    auto vk_hash = vkey_as_fields.back();
    vkey_as_fields.pop_back();

    *out_vkey = to_heap_buffer(vkey_as_fields);
    write(out_key_hash, vk_hash);
}
