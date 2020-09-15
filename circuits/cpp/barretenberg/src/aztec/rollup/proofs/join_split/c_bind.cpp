#include "c_bind.h"
#include "join_split.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::join_split;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void join_split__init_proving_key()
{
    // We know that we don't actually need any CRS to create a proving key, so just feed in a nothing.
    // Hacky, but, right now it needs *something*.
    auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
    init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__init_proving_key_from_buffer(uint8_t const* pk_buf)
{
    std::shared_ptr<waffle::ProverReferenceString> crs;
    waffle::proving_key_data pk_data;
    read(pk_buf, pk_data);
    init_proving_key(crs, std::move(pk_data));
}

WASM_EXPORT uint32_t join_split__get_new_proving_key_data(uint8_t** output)
{
    // Computing the size of the serialized key is non trivial. We know it's ~331mb.
    // Allocate a buffer large enough to hold it, and abort if we overflow.
    // This is to keep memory usage down.
    size_t total_buf_len = 350 * 1024 * 1024;
    auto raw_buf = (uint8_t*)malloc(total_buf_len);
    auto raw_buf_end = raw_buf;
    write(raw_buf_end, *get_proving_key());
    *output = raw_buf;
    auto len = static_cast<uint32_t>(raw_buf_end - raw_buf);
    if (len > total_buf_len) {
        info("Buffer overflow serializing proving key.");
        std::abort();
    }
    return len;
}

WASM_EXPORT void join_split__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__init_verification_key_from_buffer(uint8_t const* vk_buf, uint8_t const* g2x)
{
    auto crs = std::make_shared<waffle::VerifierMemReferenceString>(g2x);
    waffle::verification_key_data vk_data;
    read(vk_buf, vk_data);
    init_verification_key(crs, std::move(vk_data));
}

WASM_EXPORT uint32_t join_split__get_new_verification_key_data(uint8_t** output)
{
    auto buffer = to_buffer(*get_verification_key());
    auto raw_buf = (uint8_t*)malloc(buffer.size());
    memcpy(raw_buf, (void*)buffer.data(), buffer.size());
    *output = raw_buf;
    return static_cast<uint32_t>(buffer.size());
}

WASM_EXPORT void* join_split__new_prover(uint8_t const* join_split_buf)
{
    auto tx = from_buffer<join_split_tx>(join_split_buf);
    auto prover = new_join_split_prover(tx);
    auto heapProver = new UnrolledProver(std::move(prover));
    return heapProver;
}

WASM_EXPORT void join_split__delete_prover(void* prover)
{
    delete reinterpret_cast<Prover*>(prover);
}

WASM_EXPORT bool join_split__verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return verify_proof(pp);
}
}
