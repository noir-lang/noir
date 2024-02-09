#include "c_bind.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "poseidon2.hpp"

extern "C" {

WASM_EXPORT void poseidon_hash(bb::fr::vec_in_buf inputs_buffer, bb::fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    auto r = bb::crypto::Poseidon2<bb::crypto::Poseidon2Bn254ScalarFieldParams>::hash(to_hash);
    bb::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void poseidon_hashes(bb::fr::vec_in_buf inputs_buffer, bb::fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    const size_t numHashes = to_hash.size() / 2;
    std::vector<grumpkin::fq> results;
    size_t count = 0;
    while (count < numHashes) {
        auto r = bb::crypto::Poseidon2<bb::crypto::Poseidon2Bn254ScalarFieldParams>::hash(
            { to_hash[count * 2], to_hash[count * 2 + 1] });
        results.push_back(r);
        ++count;
    }
    write(output, results);
}
}