#include "c_bind.hpp"
#include "../pedersen_hash/pedersen.hpp"
#include "barretenberg/common/serialize.hpp"
#include "pedersen.hpp"

WASM_EXPORT void pedersen__commit(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_commit;
    read(inputs_buffer, to_commit);
    grumpkin::g1::affine_element pedersen_commitment = crypto::pedersen_commitment::commit_native(to_commit);

    serialize::write(output, pedersen_commitment);
}