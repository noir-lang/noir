// #pragma once

// namespace waffle {

// template <typename getters, typename polynomial_container> class ArithmeticKernel {
//   public:
//     inline static barretenberg::fr sum_linear_terms(polynomial_container& polynomials,
//                                                     result_container& linear_terms,
//                                                     const challenge_container& challenges)
//     {
//         const Field& alpha = getters::get_challenge(challenges, ChallengeId::ALPHA);
//         const Field& q_1 = getters::get_polynomial(polynomials, PolynomialId::Q1, i);
//         const Field& q_2 = getters::get_polynomial(polynomials, PolynomialId::Q1, i);
//         const Field& q_3 = getters::get_polynomial(polynomials, PolynomialId::Q1, i);
//         const Field& q_m = getters::get_polynomial(polynomials, PolynomialId::QM, i);
//         const Field& q_c = getters::get_polynomial(polynomials, PolynomialId::QC, i);

//         Field result = linear_terms[0] * q_m;
//         result += (linear_terms[1] * q_1);
//         result += (linear_terms[2] * q_2);
//         result += (linear_terms[3] * q_3);
//         result += (linear_terms[4] * q_c);
//         result *= alpha;
//         return result;
//     }

//     inline static void compute_linearised_terms(polynomial_container& polynomials,
//                                                 result_container& linear_terms,
//                                                 const size_t i)
//     {
//         const fr& w_1 = getters::get_polynomial(polynomials, PolynomialId::W1, 2 * i);
//         const fr& w_2 = getters::get_polynomial(polynomials, PolynomialId::W2, 2 * i);
//         const fr& w_3 = getters::get_polynomial(polynomials, PolynomialId::W3, 2 * i);

//         linear_terms[0] = w_1 * w_2;
//         linear_terms[1] = w_1;
//         linear_terms[2] = w_2;
//         linear_terms[3] = w_3;
//         linear_terms[4] = q_c;
//     }

//     inline static void apply_ inline static void compute_linearised_multiplication_scalars(
//         result_container& linear_terms, std::map<std::string, Field>& scalars, const challenge_container& challenges)
//     {
//         const Field& alpha = getters::get_challenge(challenges, ChallengeId::ALPHA);
//         const Field& linear_challenge = getters::get_challenge(challenges, ChallengeId::LINEAR_NU);
//         scalars.at("Q_M") = linear_terms[0] * alpha * linear_challenge;
//         scalars.at("Q_1") = linear_terms[1] * alpha * linear_challenge;
//         scalars.at("Q_2") = linear_terms[2] * alpha * linear_challenge;
//         scalars.at("Q_3") = linear_terms[3] * alpha * linear_challenge;
//         scalars.at("Q_C") = linear_terms[4] * alpha * linear_challenge;
//     }

//     inline static barretenberg::fr compute_non_linear_terms(polynomial_container&,
//                                                             const challenge_container&,
//                                                             const size_t)
//     {}
// };
// template <class KernelBase, class Field> class GenericProverWidget {

//   public:
//     static constexpr size_t MAX_NUM_POLYNOMIALS = 32;
//     typedef std::array<Field, MAX_NUM_POLYNOMIALS> poly_ptr_array;
//     typedef std::array<Field*, MAX_NUM_POLYNOMIALS> poly_ptr_array;
//     typedef KernelBase<get_prover_polynomial, poly_ptr_array> ProverKernel;

//     inline static Field& get_prover_polynomial(poly_ptr_array& polynomials, PolynomialId id, const size_t index)
//     {
//         return polynomials[id][index];
//     }

//     static Field compute_quotient_contribution(const Field& alpha_base, const transcript::Transcript& transcript)
//     {
//         poly_ptr_array polynomials = get_prover_polynomials();
//         poly_array challenges = get_challenges(transcript);
//         poly_array linear_terms;

//         ITERATE_OVER_DOMAIN_START(key->mid_domain);
//         ProverKernel::compute_linearized_terms(polynomials, linear_terms, i);
//         quotient_mid[i] += ProverKernel::sum_linear_terms(linear_terms, challenges);
//         quotient_mid[i] += ProverKernel::compute_non_linear_terms(polynomials, challenges, i);
//         ITERATE_OVER_DOMAIN_END;
//     }

//     static Field compute_linear_contribution(const Field& alpha_base, const transcript::Transcript& transcript)
//     {
//         poly_ptr_array polynomials = get_prover_polynomials();
//         poly_array challenges = get_challenges(transcript);
//         poly_array linear_terms;

//         ITERATE_OVER_DOMAIN_START(key->small_domain);
//         ProverKernel::compute_linearized_terms(polynomials, linear_terms, i);
//         r[i] += ProverKernel::sum_linear_terms(linear_terms, challenges);
//         ITERATE_OVER_DOMAIN_END;
//     }

//   private:
// };

// template <class Kernel, class Field> class GenericVerifierWidget {

//   public:
//     static constexpr size_t MAX_NUM_POLYNOMIALS = 32;
//     typedef std::array<Field, MAX_NUM_POLYNOMIALS> poly_array;
//     typedef KernelBase<get_verifier_polynomial, poly_array> ProverKernel;

//     inline static Field& get_verifier_polynomial(poly_array& polynomials, PolynomialId id, const size_t index)
//     {
//         return polynomials[id];
//     }

//     static Field compute_quotient_contribution(verification_key* key,
//                                                const Field& alpha,
//                                                const transcript::Transcript& transcript,
//                                                Field& t_eval,
//                                                const bool use_linerisation)
//     {
//         poly_ptr_array polynomials = get_prover_polynomials();
//         poly_array challenges = get_challenges(transcript);

//         t_eval += Kernel::compute_non_linear_terms<get_prover_polynomial, poly_ptr_array>(polynomials, challenges,
//         i);

//         if (!use_linearisation) {
//             poly_array linear_terms;
//             VerifierKernel::compute_linearized_terms(polynomials, linear_terms, i);
//             t_eval += VerifierKernel::sum_linear_terms(linear_terms, challenges);
//         }
//     }

//     static Field compute_batch_evaluation_contribution(verification_key* key,
//                                                        Field& batch_eval,
//                                                        const transcript::Transcript& transcript,
//                                                        const bool use_linearisation)
//     {
//         if (!use_linearisation) {
//             return;
//         }
//         poly_array linear_terms;
//         VerifierKernel::compute_linearized_terms(polynomials, linear_terms, i);
//         t_eval += VerifierKernel::sum_linear_terms(linear_terms, challenges);
//     }

//     static Field append_scalar_multiplication_inputs(verification_key* key,
//                                                      const Field& alpha_base,
//                                                      const Transript& transcript,
//                                                      std::vector<Group>& elements,
//                                                      std::map<std::string, Field>& scalars)
//     {
//         if (!use_linearisation) {
//             return;
//         }

//         poly_array linear_terms;
//         VerifierKernel::compute_linearized_terms(polynomials, linear_terms, i);

//         VerifierKernel::map_linear_terms_to_scalar_multiplications(linear_terms, scalars);
//     }

//   private:
// };

// }