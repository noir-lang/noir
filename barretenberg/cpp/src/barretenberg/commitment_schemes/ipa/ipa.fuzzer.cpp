#define IPA_FUZZ_TEST
#include "ipa.hpp"
#include "./mock_transcript.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

namespace bb {

// We actually only use 4, because fuzzing is very slow
constexpr size_t COMMITMENT_TEST_NUM_POINTS = 32;
using Curve = curve::Grumpkin;
std::shared_ptr<CommitmentKey<Curve>> ck;
std::shared_ptr<VerifierCommitmentKey<Curve>> vk;
/**
 * @brief Class that allows us to call internal IPA methods, because it's friendly
 *
 */
class ProxyCaller {
  public:
    template <typename Transcript>
    static void compute_opening_proof_internal(const std::shared_ptr<CommitmentKey<Curve>>& ck,
                                               const OpeningPair<Curve>& opening_pair,
                                               const Polynomial<Curve::ScalarField>& polynomial,
                                               const std::shared_ptr<Transcript>& transcript)
    {
        IPA<Curve>::compute_opening_proof_internal(ck, opening_pair, polynomial, transcript);
    }
    template <typename Transcript>
    static bool verify_internal(const std::shared_ptr<VerifierCommitmentKey<Curve>>& vk,
                                const OpeningClaim<Curve>& opening_claim,
                                const std::shared_ptr<Transcript>& transcript)
    {
        return IPA<Curve>::reduce_verify_internal(vk, opening_claim, transcript);
    }
};
} // namespace bb

/**
 * @brief Initialize SRS, commitment key, verification key
 *
 */
extern "C" void LLVMFuzzerInitialize(int*, char***)
{
    srs::init_grumpkin_crs_factory("../srs_db/ignition");
    ck = std::make_shared<CommitmentKey<Curve>>(COMMITMENT_TEST_NUM_POINTS);
    auto crs_factory = std::make_shared<srs::factories::FileCrsFactory<curve::Grumpkin>>("../srs_db/grumpkin",
                                                                                         COMMITMENT_TEST_NUM_POINTS);
    vk = std::make_shared<VerifierCommitmentKey<curve::Grumpkin>>(COMMITMENT_TEST_NUM_POINTS, crs_factory);
}

// This define is needed to make ProxyClass a friend of IPA
#define IPA_FUZZ_TEST
#include "ipa.hpp"

/**
 * @brief A fuzzer for the IPA primitive
 *
 * @details Parses the given data as a polynomial, a sequence of challenges for the transcript and the evaluation point,
 * then opens the polynomial with IPA and verifies that the opening was correct
 */
extern "C" int LLVMFuzzerTestOneInput(const unsigned char* data, size_t size)
{
    using Fr = grumpkin::fr;
    using Polynomial = Polynomial<Fr>;
    // We need data
    if (size == 0) {
        return 0;
    }
    // Get the logarighmic size of polynomial
    const auto log_size = static_cast<size_t>(data[0]);
    // More than 4 is so bad
    if (log_size == 0 || log_size > 2) {
        return 0;
    }
    const auto* offset = data + 1;
    const auto num_challenges = log_size + 1;
    // How much data do we need?
    // Challenges: sizeof(uint256_t) * num_challenges + 1 for montgomery switch
    // Polynomial: sizeof(uint256_t) * size + 1 per size/8
    // Eval x: sizeof(uint256_t) + 1
    const size_t polynomial_size = (1 << log_size);
    // Bytes controlling montgomery switching for polynomial coefficients
    const size_t polynomial_control_bytes = (polynomial_size < 8 ? 1 : polynomial_size / 8);
    const size_t expected_size =
        sizeof(uint256_t) * (num_challenges + polynomial_size + 1) + 3 + polynomial_control_bytes;
    if (size < expected_size) {
        return 0;
    }

    // Initialize transcript
    auto transcript = std::make_shared<MockTranscript>();

    std::vector<uint256_t> challenges(num_challenges);
    // Get the byte, where bits control if we parse challenges in montgomery form or not
    const auto control_byte = offset[0];
    offset++;
    // Get challenges one by one
    for (size_t i = 0; i < num_challenges; i++) {
        auto challenge = *(uint256_t*)(offset);

        if ((control_byte >> i) & 1) {
            // If control byte says so, parse the value from input as if it's internal state of the field (already
            // converted to montgomery). This allows modifying the state directly
            auto field_challenge = Fr(challenge);

            challenge = field_challenge.from_montgomery_form();
        }
        // Challenges can't be zero
        if (Fr(challenge).is_zero()) {
            return 0;
        }
        challenges[i] = challenge;
        offset += sizeof(uint256_t);
    }

    // Put challenges into the transcript
    transcript->initialize(challenges);

    // Parse polynomial
    std::vector<uint256_t> polynomial_coefficients(polynomial_size);
    for (size_t i = 0; i < polynomial_size; i++) {
        polynomial_coefficients[i] = *(uint256_t*)(offset);
        offset += sizeof(uint256_t);
    }
    Polynomial poly(polynomial_size);

    // Convert from montgomery if the appropriate bit is set
    for (size_t i = 0; i < polynomial_size; i++) {
        auto b = offset[i / 8];

        poly[i] = polynomial_coefficients[i];
        if ((b >> (i % 8)) & 1) {
            poly[i].self_from_montgomery_form();
        }
    }

    offset += polynomial_control_bytes;
    // Parse the x we are evaluating on
    auto x = Fr(*(uint256_t*)offset);
    offset += sizeof(uint256_t);
    if ((offset[0] & 1) != 0) {
        x.self_from_montgomery_form();
    }
    auto const opening_pair = OpeningPair<Curve>{ x, poly.evaluate(x) };
    auto const opening_claim = OpeningClaim<Curve>{ opening_pair, ck->commit(poly) };
    ProxyCaller::compute_opening_proof_internal(ck, opening_pair, poly, transcript);

    // Reset challenge indices
    transcript->reset_indices();

    // Should verify
    if (!ProxyCaller::verify_internal(vk, opening_claim, transcript)) {
        return 1;
    }
    return 0;
}