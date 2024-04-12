#include "c_bind.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "poseidon2.hpp"
#include "poseidon2_permutation.hpp"

using namespace bb;

WASM_EXPORT void poseidon2_hash(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<fr> to_hash;
    read(inputs_buffer, to_hash);
    auto r = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(to_hash);
    fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void poseidon2_hashes(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<fr> to_hash;
    read(inputs_buffer, to_hash);
    const size_t numHashes = to_hash.size() / 2;
    std::vector<fr> results;
    size_t count = 0;
    while (count < numHashes) {
        auto r = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(
            { to_hash[count * 2], to_hash[count * 2 + 1] });
        results.push_back(r);
        ++count;
    }
    write(output, results);
}

WASM_EXPORT void poseidon2_permutation(fr::vec_in_buf inputs_buffer, fr::vec_out_buf output)
{
    using Permutation = crypto::Poseidon2Permutation<crypto::Poseidon2Bn254ScalarFieldParams>;

    // Serialise input vector.
    std::vector<fr> to_permute;
    read(inputs_buffer, to_permute);

    // Copy input vector into Permutation::State (which is an std::array).
    Permutation::State input_state;
    std::copy(to_permute.begin(), to_permute.end(), input_state.data());

    Permutation::State results_array = Permutation::permutation(input_state);

    const std::vector<fr> results(results_array.begin(), results_array.end());
    *output = to_heap_buffer(results);
}
