#include <common/serialize.hpp>
#include <crypto/blake2s/blake2s.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

using namespace barretenberg;

inline fr compute_nullifier(grumpkin::g1::affine_element const& note_commitment, uint32_t index)
{
    std::vector<uint8_t> buf;
    write(buf, note_commitment.x);
    write(buf, fr(index));
    auto blake_result = blake2::blake2s(buf);
    return from_buffer<fr>(blake_result);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup