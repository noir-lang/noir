#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "c_bind.hpp"
#include "pedersen.hpp"

extern "C" {

WASM_EXPORT void pedersen_hash_init() {}

WASM_EXPORT void pedersen_hash_with_hash_index(uint8_t const* inputs_buffer,
                                               uint32_t const* hash_index,
                                               uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_hash::hash(to_compress, ntohl(*hash_index));
    barretenberg::fr::serialize_to_buffer(r, output);
}
}