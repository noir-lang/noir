
#include "kate.hpp"

#include <common/mem.hpp>
#include <gtest/gtest.h>

#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <polynomials/polynomial.hpp>

#include <srs/reference_string/reference_string.hpp>
#include <plonk/composer/composer_base.hpp>
#include <plonk/proof_system/types/program_settings.hpp>
#include <proof_system/work_queue/work_queue.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

using namespace barretenberg;
using namespace waffle;

struct Bn254 {
    typedef barretenberg::fq Fq;
    typedef barretenberg::fr Fr;
    typedef barretenberg::g1 G1;
};
typedef KateCommitmentSchemeData<Bn254> CommitmentSchemeData;

static std::array<barretenberg::fr, 10> challenges{ fr::random_element(), fr::random_element(), fr::random_element(),
                                                    fr::random_element(), fr::random_element(), fr::random_element(),
                                                    fr::random_element(), fr::random_element(), fr::random_element(),
                                                    fr::random_element() };

class MockChallengeGenerator {
  public:
    fr generate_challenge(const std::vector<fr>&)
    {
        auto result = challenges[count % 10];
        count++;
        return result;
    }
    size_t count = 0;
};

typedef Kate<KateCommitmentSchemeData<Bn254>, MockChallengeGenerator> KateBn254;

// TEST(kate, open)
// {
//     const size_t n = 16;

//     std::cout << "a" << std::endl;
//     auto file_crs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
//     auto crs = file_crs->get_prover_crs(n);
//     std::cout << "b" << std::endl;

//     polynomial poly(n, n);
//     for (size_t i = 0; i < 16; ++i) {
//         poly[i] = fr::random_element();
//     }

//     std::cout << "c" << std::endl;
//     const std::vector<g1::affine_element> commitments = KateBn254::commit({ &poly[0] }, n, crs);

//     transcript::StandardTranscript inp_tx = transcript::StandardTranscript({});
//     waffle::KateCommitmentScheme<turbo_settings> newKate;
//     auto circuit_proving_key = std::make_shared<proving_key>(n, 0, crs);
//     work_queue queue(circuit_proving_key.get(), nullptr, &inp_tx);
//     newKate.commit(&poly[0], "F_COMM", 0, queue);
//     queue.process_queue();

//     auto expected = inp_tx.get_group_element("F_COMM");

//     EXPECT_EQ(commitments[0], expected);
// }

// These tests were never working in sumcheck work.
// TEST(kate, kate_commit_open_verify_one_poly)
// {
//     const size_t n = 16;

//     auto file_crs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
//     auto crs = file_crs->get_prover_crs(n);
//     auto verifier_crs = file_crs->get_verifier_crs();

//     polynomial poly(n, n);
//     for (size_t i = 0; i < 16; ++i) {
//         poly[i] = fr::random_element();
//     }

//     fr z = fr::random_element();

//     fr eval = poly.evaluate(z, n);

//     const std::vector<g1::affine_element> commitments = KateBn254::commit({ &poly[0] }, n, crs);

//     KateBn254::OpeningSchema opening_schema{ .polynomials = { &poly[0] },
//                                              .shifted_polynomials = {},
//                                              .commitments = commitments,
//                                              .variables = { { z } },
//                                              .evaluations = { { eval } },
//                                              .shifted_evaluations = {},
//                                              .n = n };

//     MockChallengeGenerator prover_challenges;
//     const auto opening_proof = KateBn254::batch_open(opening_schema, crs, prover_challenges);

//     MockChallengeGenerator verifier_challenges;
//     const bool verified = KateBn254::batch_verify(opening_proof, verifier_crs, verifier_challenges);

//     EXPECT_EQ(verified, true);
// }

// TEST(kate, kate_commit_open_verify_many_poly_many_vars)
// {
//     const size_t num_polys = 9;
//     const size_t num_openings = 7;
//     const size_t n = 8;

//     auto file_crs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
//     auto crs = file_crs->get_prover_crs(n);
//     auto verifier_crs = file_crs->get_verifier_crs();

//     std::vector<polynomial> polys(num_polys);
//     std::vector<fr> evaluation_points;
//     std::vector<std::vector<fr>> polynomial_evaluations;

//     // n.b. when working with pointers as pointers to arrays, the underlying data cannot move!
//     // this makes them a brittle construct.
//     // Atm the commitment scheme spec represents polynomials as Fr* arrays...
//     // This is because the current `polynomial` class is itself very brittle and ideally needs a big refactor,
//     // it also only works for barretenberg::fr which is not sufficient anymore (i.e. we need grumpkin::fr support
//     too) std::vector<fr*> poly_pointers; for (size_t i = 0; i < num_openings; ++i) {
//         evaluation_points.push_back(fr::random_element());
//     }

//     for (size_t j = 0; j < num_polys; ++j) {
//         polys[j] = polynomial(n, n);
//         auto& poly = polys[j];

//         // polynomial poly(n, n);
//         for (size_t i = 0; i < n; ++i) {
//             poly[i] = fr::random_element();
//         }
//         std::vector<fr> evaluations;
//         for (size_t i = 0; i < num_openings; ++i) {
//             evaluations.push_back(poly.evaluate(evaluation_points[i], n));
//         }
//         polynomial_evaluations.push_back(evaluations);

//         poly_pointers.push_back(&(polys[j][0]));
//     }

//     const std::vector<g1::affine_element> commitments = KateBn254::commit(poly_pointers, n, crs);

//     KateBn254::OpeningSchema opening_schema{ .polynomials = poly_pointers,
//                                              .shifted_polynomials = {},
//                                              .commitments = commitments,
//                                              .variables = { evaluation_points },
//                                              .evaluations = polynomial_evaluations,
//                                              .shifted_evaluations = {},
//                                              .n = n };

//     MockChallengeGenerator prover_challenges;

//     const auto opening_proof = KateBn254::batch_open(opening_schema, crs, prover_challenges);

//     MockChallengeGenerator verifier_challenges;
//     const bool verified = KateBn254::batch_verify(opening_proof, verifier_crs, verifier_challenges);

//     EXPECT_EQ(verified, true);
// }

//     // generate random polynomial F(X) = coeffs
//     size_t n = 256;
//     std::vector<fr> coeffs(n);
//     for (size_t i = 0; i < n; ++i) {
//         coeffs[i] = fr::random_element();
//     }
//     std::vector<fr> W(coeffs.begin(), coeffs.end());

//     // generate random evaluation point z
//     fr z = fr::random_element();

//     // compute opening polynomial W(X), and evaluation f = F(z)
//     transcript::StandardTranscript inp_tx = transcript::StandardTranscript({});
//     waffle::KateCommitmentScheme<turbo_settings> newKate;

//     // std::shared_ptr<ReferenceStringFactory> crs_factory = (new FileReferenceStringFactory("../srs_db"));
//     auto file_crs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
//     auto crs = file_crs->get_prover_crs(n);
//     auto circuit_proving_key = std::make_shared<proving_key>(n, 0, crs);
//     work_queue queue(circuit_proving_key.get(), nullptr, &inp_tx);

//     newKate.commit(&coeffs[0], "F_COMM", 0, queue);
//     queue.process_queue();

//     fr y = fr::random_element();
//     fr f_y = polynomial_arithmetic::evaluate(&coeffs[0], y, n);
//     fr f = polynomial_arithmetic::evaluate(&coeffs[0], z, n);

//     newKate.compute_opening_polynomial(&coeffs[0], &W[0], z, n, "W_COMM", fr(0), queue);
//     queue.process_queue();

//     // check if W(y)(y - z) = F(y) - F(z)
//     fr w_y = polynomial_arithmetic::evaluate(&W[0], y, n);
//     fr y_minus_z = y - z;
//     fr f_y_minus_f = f_y - f;

//     EXPECT_EQ(w_y * y_minus_z, f_y_minus_f);

// }

// TEST(commitment_scheme, kate_batch_open)
// {
//     // generate random evaluation points [z_1, z_2, ...]
//     size_t t = 8;
//     std::vector<fr> z_points(t);
//     for (size_t k = 0; k < t; ++k) {
//         z_points[k] = fr::random_element();
//     }

//     // generate random polynomials F(X) = coeffs
//     //
//     // z_1 -> [F_{1,1},  F_{1,2},  F_{1, 3},  ...,  F_{1, m}]
//     // z_2 -> [F_{2,1},  F_{2,2},  F_{2, 3},  ...,  F_{2, m}]
//     // ...
//     // z_t -> [F_{t,1},  F_{t,2},  F_{t, 3},  ...,  F_{t, m}]
//     //
//     // Note that each polynomial F_{k, j} \in F^{n}
//     //
//     size_t n = 64;
//     size_t m = 4;
//     std::vector<fr> coeffs(n * m * t);
//     for (size_t k = 0; k < t; ++k) {
//         for (size_t j = 0; j < m; ++j) {
//             for (size_t i = 0; i < n; ++i) {
//                 coeffs[k * (m * n) + j * n + i] = fr::random_element();
//             }
//         }
//     }

//     // setting up the Kate commitment scheme class
//     transcript::StandardTranscript inp_tx = transcript::StandardTranscript({});
//     waffle::KateCommitmentScheme<turbo_settings> newKate;

//     auto file_crs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
//     auto crs = file_crs->get_prover_crs(n);
//     auto circuit_proving_key = std::make_shared<proving_key>(n, 0, crs);
//     work_queue queue(circuit_proving_key.get(), nullptr, &inp_tx);

//     // commit to individual polynomials
//     for (size_t k = 0; k < t; ++k) {
//         for (size_t j = 0; j < m; ++j) {
//             newKate.commit(&coeffs[k * m * n + j * n], "F_{" + std::to_string(k + 1) + ", " + std::to_string(j + 1) +
//             "}", 0, queue);
//         }
//     }
//     queue.process_queue();

//     // create random challenges, tags and item_constants
//     std::vector<fr> challenges(t);
//     std::vector<std::string> tags(t);
//     std::vector<fr> item_constants(t);
//     for (size_t k = 0; k < t; ++k) {
//         challenges[k] = fr::random_element();
//         tags[k] = "W_" + std::to_string(k + 1);
//         item_constants[k] = fr(0);
//     }

//     // compute opening polynomials W_1, W_2, ..., W_t
//     std::vector<fr> W(n * t);
//     newKate.generic_batch_open(&coeffs[0], &W[0], m, &z_points[0], t, &challenges[0], n, &tags[0],
//     &item_constants[0], queue); queue.process_queue();

//     // check if W_{k}(y) * (y - z_k) = \sum_{j} challenge[k]^{j - 1} * [F_{k, j}(y) - F_{k, j}(z_k)]
//     fr y = fr::random_element();
//     for (size_t k = 0; k < t; ++k) {

//         // compute lhs
//         fr W_k_at_y = polynomial_arithmetic::evaluate(&W[k * n], y, n);
//         fr y_minus_z_k = y - z_points[k];
//         fr lhs = W_k_at_y * y_minus_z_k;

//         fr challenge_pow = fr(1);
//         fr rhs = fr(0);
//         for (size_t j = 0; j < m; ++j) {

//             // compute evaluations of source polynomials at y and z_points
//             fr f_kj_at_y = polynomial_arithmetic::evaluate(&coeffs[k * m * n  +  j * n], y, n);
//             fr f_kj_at_z = polynomial_arithmetic::evaluate(&coeffs[k * m * n  +  j * n], z_points[k], n);

//             // compute rhs
//             fr f_term = f_kj_at_y - f_kj_at_z;
//             rhs += challenge_pow * f_term;
//             challenge_pow *= challenges[k];
//         }

//         EXPECT_EQ(lhs, rhs);
//     }
// }
