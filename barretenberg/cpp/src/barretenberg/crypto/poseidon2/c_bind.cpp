#include "c_bind.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "poseidon2.hpp"

using namespace bb;

WASM_EXPORT void poseidon_hash(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    auto r = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(to_hash);
    fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void poseidon_hashes(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    const size_t numHashes = to_hash.size() / 2;
    std::vector<grumpkin::fq> results;
    size_t count = 0;
    while (count < numHashes) {
        auto r = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(
            { to_hash[count * 2], to_hash[count * 2 + 1] });
        results.push_back(r);
        ++count;
    }
    write(output, results);
}