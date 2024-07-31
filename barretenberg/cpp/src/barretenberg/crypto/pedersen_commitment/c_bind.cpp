#include "c_bind.hpp"
#include "../pedersen_hash/pedersen.hpp"
#include "barretenberg/common/serialize.hpp"
#include "pedersen.hpp"

using namespace bb;

WASM_EXPORT void pedersen_commit(fr::vec_in_buf inputs_buffer,
                                 uint32_t const* ctx_index,
                                 grumpkin::g1::affine_element::out_buf output)
{
    std::vector<grumpkin::fq> to_commit;
    read(inputs_buffer, to_commit);
    crypto::GeneratorContext<curve::Grumpkin> ctx;
    ctx.offset = static_cast<size_t>(ntohl(*ctx_index));
    grumpkin::g1::affine_element pedersen_commitment = crypto::pedersen_commitment::commit_native(to_commit, ctx);

    write(output, pedersen_commitment);
}
