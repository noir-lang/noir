#include <cstdint>
#include <sstream>

#include "../mock/mock_circuit.hpp"
#include "barretenberg/common/container.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "c_bind.h"
#include "compute_signing_data.hpp"
#include "join_split.hpp"

using namespace barretenberg;
using namespace join_split_example::proofs::join_split;

WASM_EXPORT void join_split__init_proving_key(bool mock)
{
    init_proving_key(barretenberg::srs::get_crs_factory(), mock);
}

// WASM_EXPORT void join_split__init_proving_key_from_buffer(uint8_t const* pk_buf)
// {
//     std::shared_ptr<barretenberg::srs::factories::ProverCrs> crs;
//     plonk::proving_key_data pk_data;
//     read(pk_buf, pk_data);
//     init_proving_key(crs, std::move(pk_data));
// }

WASM_EXPORT void join_split__release_key()
{
    release_proving_key();
}

WASM_EXPORT uint32_t join_split__get_new_proving_key_data(uint8_t** output)
{
    // Computing the size of the serialized key is non trivial. We know it's ~331mb.
    // Allocate a buffer large enough to hold it, and abort if we overflow.
    // This is to keep memory usage down.

    auto proving_key = get_proving_key();
    auto buffer = to_buffer(*proving_key);
    auto raw_buf = (uint8_t*)malloc(buffer.size());
    memcpy(raw_buf, (void*)buffer.data(), buffer.size());
    *output = raw_buf;

    return static_cast<uint32_t>(buffer.size());
}

WASM_EXPORT void join_split__init_verification_key(void* /*unused*/, uint8_t const* /*unused*/)
{
    init_verification_key(barretenberg::srs::get_crs_factory());
}

// WASM_EXPORT void join_split__init_verification_key_from_buffer(uint8_t const* vk_buf, uint8_t const* g2x)
// {
//     auto crs = std::make_shared<proof_system::VerifierMemReferenceString>(g2x);
//     plonk::verification_key_data vk_data;
//     read(vk_buf, vk_data);
//     init_verification_key(crs, std::move(vk_data));
// }

WASM_EXPORT uint32_t join_split__get_new_verification_key_data(uint8_t** output)
{
    auto buffer = to_buffer(*get_verification_key());
    auto raw_buf = (uint8_t*)malloc(buffer.size());
    memcpy(raw_buf, (void*)buffer.data(), buffer.size());
    *output = raw_buf;
    return static_cast<uint32_t>(buffer.size());
}

WASM_EXPORT void join_split__compute_signing_data(uint8_t const* join_split_tx_buf, uint8_t* output)
{
    auto tx = from_buffer<join_split_tx>(join_split_tx_buf);
    auto signing_data = compute_signing_data(tx);
    barretenberg::fr::serialize_to_buffer(signing_data, output);
}

WASM_EXPORT void* join_split__new_prover(uint8_t const* join_split_buf, bool mock)
{
    auto tx = from_buffer<join_split_tx>(join_split_buf);
    auto prover = new_join_split_prover(tx, mock);
    auto heapProver = new join_split_example::Prover(std::move(prover));
    return heapProver;
}

WASM_EXPORT void join_split__delete_prover(void* prover)
{
    delete reinterpret_cast<join_split_example::Prover*>(prover);
}

WASM_EXPORT bool join_split__verify_proof(uint8_t* proof, uint32_t length)
{
    plonk::proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return verify_proof(pp);
}
