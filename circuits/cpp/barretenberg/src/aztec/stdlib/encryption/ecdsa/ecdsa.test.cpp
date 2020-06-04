#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "ecdsa.hpp"

#include <crypto/ecdsa/ecdsa.hpp>

#include <common/test.hpp>
#include <ecc/curves/secp256r1/secp256r1.hpp>

using namespace barretenberg;
using namespace plonk;

namespace plonk {
namespace stdlib {
namespace secp256r {
typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FqParams> fq;
typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FrParams> fr;
typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, secp256r1::g1> g1;

} // namespace secp256r
} // namespace stdlib
} // namespace plonk
typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

stdlib::secp256r::g1 convert_inputs(waffle::TurboComposer* ctx, const secp256r1::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);

    stdlib::secp256r::fq x(witness_t(ctx, barretenberg::fr(x_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
                           witness_t(ctx,
                                     barretenberg::fr(x_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                                   stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));
    stdlib::secp256r::fq y(witness_t(ctx, barretenberg::fr(y_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
                           witness_t(ctx,
                                     barretenberg::fr(y_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                                   stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));

    return stdlib::secp256r::g1(x, y);
}

stdlib::secp256r::fr convert_inputs(waffle::TurboComposer* ctx, const barretenberg::fr& scalar)
{
    uint256_t scalar_u256(scalar);

    stdlib::secp256r::fr x(
        witness_t(ctx, barretenberg::fr(scalar_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
        witness_t(ctx,
                  barretenberg::fr(scalar_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                     stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));

    return x;
}

HEAVY_TEST(stdlib_ecdsa, verify_signature)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<secp256r1::fr, secp256r1::g1> account;
    account.private_key = secp256r1::fr::random_element();
    account.public_key = secp256r1::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(message_string,
                                                                                                      account);

    bool first_result = crypto::ecdsa::verify_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    stdlib::secp256r::g1 public_key = convert_inputs(&composer, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());

    stdlib::ecdsa::signature<waffle::TurboComposer> sig{ stdlib::byte_array<waffle::TurboComposer>(&composer, rr),
                                                         stdlib::byte_array<waffle::TurboComposer>(&composer, ss) };

    stdlib::byte_array message(&composer, message_string);

    stdlib::bool_t<waffle::TurboComposer> signature_result = stdlib::ecdsa::
        verify_signature<waffle::TurboComposer, stdlib::secp256r::fq, stdlib::secp256r::fr, stdlib::secp256r::g1>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
