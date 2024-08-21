#pragma once
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "sumcheck_round.hpp"

namespace bb {

/*! \brief The implementation of the sumcheck Prover for statements of the form \f$\sum_{\vec \ell \in \{0,1\}^d}
pow_{\beta}(\vec \ell) \cdot F \left(P_1(\vec \ell),\ldots, P_N(\vec \ell) \right)  = 0 \f$ for multilinear polynomials
\f$P_1, \ldots, P_N \f$.

   \details
 \section SumcheckProverNotation Notation and Setup

 \subsection SumcheckProverObtainingPolynomials Obtaining Prover/Honk Polynomials
 The Sumcheck is applied to  multivariate polynomials
\f$P_1, \ldots, P_N\f$ that are specidied by \p Flavor. Namely, \ref prove "prove method" obtains \p full_polynomials by
reference from \p Flavor 's \ref ProverPolynomials "prover polynomials". In particular, their number \f$N\f$ is
specified by the \p Flavor.

 ### Sumcheck Relation
 Given multilinear polynomials \f$ P_1,\ldots, P_N \in \mathbb{F}[X_0,\ldots, X_{d-1}] \f$ and a relation \f$ F \f$
which is a polynomial in \f$ N \f$ variables, we use Sumcheck over the polynomial
 * \f{align}{
    \tilde{F}
    (X_0,\ldots, X_{d-1}) =
    pow_{\beta}(X_0,\ldots, X_{d-1}) \cdot F\left( P_1 (X_0,\ldots, X_{d-1}), \ldots, P_N (X_0,\ldots, X_{d-1}) \right)
    \f}
to establish that \f$ F(P_1(\vec \ell),\ldots, P_N(\vec \ell) ) = 0 \f$, i.e. that \f$ F \f$ is satisfied, at every
point of \f$\{0,1\}^d\f$.

 In the implementation, the relation polynomial \f$ F \f$ is determined by \p Flavor::Relations which is fed to \ref
bb::SumcheckProverRound "Sumcheck Round Prover".

 ## Input and Parameters
 The following constants are used:
 - \f$ d \f$ \ref multivariate_d "the number of variables" in the multilinear polynomials
 - \f$ n \f$ \ref multivariate_n "the size of the hypercube", i.e. \f$ 2^d\f$.
 - \f$ D = \f$  \ref bb::SumcheckProverRound< Flavor >::BATCHED_RELATION_PARTIAL_LENGTH "total degree of"
\f$\tilde{F}\f$ as a polynomial in \f$P_1,\ldots, P_N\f$ <b> incremented by </b> 1.


 ## Honk Polynomials and Partially Evaluated Polynomials

 Given \f$ N \f$ Honk \ref ProverPolynomials "Prover Polynomials" \f$ P_1, \ldots, P_N \f$, i.e. multilinear polynomials
in \f$ d \f$ variables.

### Round 0
At initialization, \ref ProverPolynomials "Prover Polynomials"
are submitted by reference into \p full_polynomials, which is a two-dimensional array with \f$N\f$ columns and \f$2^d\f$
rows, whose entries are defined as follows \f$\texttt{full_polynomials}_{i,j} = P_j(\vec i) \f$. Here, \f$ \vec i \in
\{0,1\}^d \f$ is identified with the binary representation of the integer \f$ 0 \leq i \leq 2^d-1 \f$.

When the first challenge \f$ u_0 \f$ is computed, the method \ref partially_evaluate "partially evaluate" takes as input
\p full_polynomials and populates  \ref partially_evaluated_polynomials "a new book-keeping table" denoted by
\f$\texttt{partially_evaluated_polynomials} \f$. Its \f$ n/2 = 2^{d-1} \f$ rows will represent the evaluations
\f$ P_i(u_0, X_1, ..., X_{d-1}) \f$, which are multilinear polynomials in \f$ d-1 \f$ variables.


More precisely, it is a table with \f$ 2^{d-1} \f$ rows and \f$ N \f$ columns, such that
    \f{align}{ \texttt{partially_evaluated_polynomials}_{i,j} = &\ P_j(0, i_1,\ldots, i_{d-1}) + u_0 \cdot
(P_j(1,i_1,\ldots, i_{d-1})) - P_j(0, i_1,\ldots, i_{d-1})) \\ = &\ \texttt{full_polynomials}_{2 i,j} + u_0 \cdot
(\texttt{full_polynomials}_{2i+1,j} - \texttt{full_polynomials}_{2 i,j}) \f}

### Updating Partial Evaluations in Subsequent Rounds
In Round \f$ i < d-1\f$, \ref partially_evaluate "partially evaluate" updates the first \f$ 2^{d-1 - i} \f$ rows of
\f$\texttt{partially_evaluated_polynomials}\f$ with the evaluations \f$ P_1(u_0,\ldots, u_i, \vec \ell),\ldots,
P_N(u_0,\ldots, u_i, \vec \ell)\f$ for \f$\vec \ell \in \{0,1\}^{d-1-i}\f$.
The details are specified in \ref partially_evaluate "the corresponding docs."

### Final Step
After computing the last challenge \f$ u_{d-1} \f$ in Round \f$ d-1 \f$ and updating \f$
\texttt{partially_evaluated_polynomials} \f$, the prover looks into the 'top' row of the table containing evaluations
\f$P_1(u_0,\ldots, u_{d-1}), \ldots, P_N(u_0,\ldots, u_{d-1})\f$ and concatenates these values with the last challenge
to the transcript.

## Round Univariates

\subsubsection SumcheckProverContributionsofPow Contributions of PowPolynomial

 * Let \f$ \vec \beta = (\beta_0,\ldots, \beta_{d-1}) \in \mathbb{F}\f$ be a vector of challenges.
 *
 * In Round \f$i\f$, a univariate polynomial \f$ \tilde S^{i}(X_{i}) \f$ for the relation defined by \f$ \tilde{F}(X)\f$
is computed as follows. First, we introduce notation
 - \f$ c_i = pow_{\beta}(u_0,\ldots, u_{i-1}) \f$
 - \f$ T^{i}( X_i ) =  \sum_{ \ell = 0} ^{2^{d-i-1}-1} \beta_{i+1}^{\ell_{i+1}} \cdot \ldots \cdot
\beta_{d-1}^{\ell_{d-1}} \cdot S^i_{\ell}( X_i )  \f$
 - \f$ S^i_{\ell} (X_i) = F \left(P_1(u_0,\ldots, u_{i-1}, X_i, \vec \ell), \ldots,  P_1(u_0,\ldots, u_{i-1}, X_i, \vec
\ell) \right) \f$

 As explained in \ref bb::PowPolynomial "PowPolynomial",
 \f{align}{
    \tilde{S}^{i}(X_i) =  \sum_{ \ell = 0} ^{2^{d-i-1}-1}   pow^i_\beta ( X_i, \ell_{i+1}, \ldots, \ell_{d-1} ) \cdot
S^i_{\ell}( X_i ) = c_i\cdot ( (1−X_i) + X_i\cdot \beta_i ) \cdot \sum_{\ell = 0}^{2^{d-i-1}-1} \beta_{i+1}^{\ell_{i+1}}
\cdot \ldots \cdot \beta_{d-1}^{\ell_{d-1}} \cdot S^{i}_{\ell}( X_i ). \f}
 *
### Computing Round Univariates
The evaluations of the round univariate \f$ \tilde{S}^i \f$ over the domain \f$0,\ldots, D \f$ are obtained by the
method \ref bb::SumcheckProverRound< Flavor >::compute_univariate "compute_univariate". The
implementation consists of the following sub-methods:

 - \ref bb::SumcheckProverRound::extend_edges "Extend evaluations" of linear univariate
polynomials \f$ P_j(u_0,\ldots, u_{i-1}, X_i, \vec \ell) \f$ to the domain \f$0,\ldots, D\f$.
 - \ref bb::SumcheckProverRound::accumulate_relation_univariates "Accumulate per-relation contributions" of the extended
polynomials to \f$ T^i(X_i)\f$
 - \ref bb::SumcheckProverRound::extend_and_batch_univariates "Extend and batch the subrelation contibutions"
multiplying by the constants \f$c_i\f$ and the evaluations of \f$ ( (1−X_i) + X_i\cdot \beta_i ) \f$.
## Transcript Operations
After computing Round univariates and adding them to the transcript, the prover generates round challenge by hashing the
transcript. These operations are taken care of by \ref bb::BaseTranscript "Transcript Class" methods.
## Output
The Sumcheck output is specified by \ref bb::SumcheckOutput< Flavor >.
 */
template <typename Flavor> class SumcheckProver {

  public:
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using PartiallyEvaluatedMultivariates = typename Flavor::PartiallyEvaluatedMultivariates;
    using ClaimedEvaluations = typename Flavor::AllValues;

    using Transcript = typename Flavor::Transcript;
    using Instance = ProverInstance_<Flavor>;
    using RelationSeparator = typename Flavor::RelationSeparator;
    /**
     * @brief The total algebraic degree of the Sumcheck relation \f$ F \f$ as a polynomial in Prover Polynomials
     * \f$P_1,\ldots, P_N\f$.
     */
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = Flavor::MAX_PARTIAL_RELATION_LENGTH;

    // this constant specifies the number of coefficients of libra polynomials, and evaluations of round univariate
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
    // Specify the number of all witnesses including shifts and derived witnesses from flavors that have ZK,
    // otherwise, set this constant to 0
    static constexpr size_t NUM_ALL_WITNESS_ENTITIES = Flavor::NUM_ALL_WITNESS_ENTITIES;
    /**
     * @brief The size of the hypercube, i.e. \f$ 2^d\f$.
     *
     */

    using SumcheckRoundUnivariate = typename bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>;
    using EvaluationMaskingTable =
        std::array<bb::Univariate<FF, MAX_PARTIAL_RELATION_LENGTH>, NUM_ALL_WITNESS_ENTITIES>;
    const size_t multivariate_n;
    /**
     * @brief The number of variables
     *
     */
    const size_t multivariate_d;
    using EvalMaskingScalars = std::array<FF, NUM_ALL_WITNESS_ENTITIES>;
    // Define the length of Libra Univariates. For non-ZK Flavors: set to 0.
    static constexpr size_t LIBRA_UNIVARIATES_LENGTH = Flavor::HasZK ? Flavor::BATCHED_RELATION_PARTIAL_LENGTH : 0;
    using LibraUnivariates = std::vector<Univariate<FF, LIBRA_UNIVARIATES_LENGTH>>;

    std::shared_ptr<Transcript> transcript;
    SumcheckProverRound<Flavor> round;
    // Declare a container for ZK Sumcheck data
    ZKSumcheckData<Flavor> zk_sumcheck_data;

    /**
    *
    * @brief Container for partially evaluated Prover Polynomials at a current challenge. Upon computing challenge \f$
    u_i \f$, the first \f$2^{d-1-i}\f$ rows are updated using \ref bb::SumcheckProver< Flavor >::partially_evaluate
    "partially evaluate" method.
    *
    * NOTE: With ~40 columns, prob only want to allocate 256 EdgeGroup's at once to keep stack under 1MB?
    * TODO(#224)(Cody): might want to just do C-style multidimensional array? for guaranteed adjacency?
    */
    PartiallyEvaluatedMultivariates partially_evaluated_polynomials;
    // prover instantiates sumcheck with circuit size and a prover transcript
    SumcheckProver(size_t multivariate_n, const std::shared_ptr<Transcript>& transcript)
        : multivariate_n(multivariate_n)
        , multivariate_d(numeric::get_msb(multivariate_n))
        , transcript(transcript)
        , round(multivariate_n)
        , partially_evaluated_polynomials(multivariate_n){};

    /**
     * @brief Compute round univariate, place it in transcript, compute challenge, partially evaluate. Repeat
     * until final round, then get full evaluations of prover polynomials, and place them in transcript.
     */
    SumcheckOutput<Flavor> prove(std::shared_ptr<Instance> instance)
    {
        return prove(instance->proving_key.polynomials,
                     instance->relation_parameters,
                     instance->alphas,
                     instance->gate_challenges);
    };

    /**
     * @brief Compute round univariate, place it in transcript, compute challenge, partially evaluate. Repeat
     * until final round, then get full evaluations of prover polynomials, and place them in transcript.
     * @details See Detailed description of \ref bb::SumcheckProver< Flavor > "Sumcheck Prover <Flavor>.
     * @param full_polynomials Container for ProverPolynomials
     * @param relation_parameters
     * @param alpha Batching challenge for subrelations.
     * @param gate_challenges
     * @return SumcheckOutput
     */
    SumcheckOutput<Flavor> prove(ProverPolynomials& full_polynomials,
                                 const bb::RelationParameters<FF>& relation_parameters,
                                 const RelationSeparator alpha,
                                 const std::vector<FF>& gate_challenges)
    {
        // In case the Flavor has ZK, we populate sumcheck data structure with randomness, compute correcting term for
        // the total sum, etc.
        if constexpr (Flavor::HasZK) {
            setup_zk_sumcheck_data(zk_sumcheck_data);
        };

        bb::PowPolynomial<FF> pow_univariate(gate_challenges);
        pow_univariate.compute_values(multivariate_d);

        std::vector<FF> multivariate_challenge;
        multivariate_challenge.reserve(multivariate_d);
        size_t round_idx = 0;
        // In the first round, we compute the first univariate polynomial and populate the book-keeping table of
        // #partially_evaluated_polynomials, which has \f$ n/2 \f$ rows and \f$ N \f$ columns. When the Flavor has ZK,
        // compute_univariate also takes into account the zk_sumcheck_data.
        auto round_univariate = round.compute_univariate(
            round_idx, full_polynomials, relation_parameters, pow_univariate, alpha, zk_sumcheck_data);
        // Place the evaluations of the round univariate into transcript.
        transcript->send_to_verifier("Sumcheck:univariate_0", round_univariate);
        FF round_challenge = transcript->template get_challenge<FF>("Sumcheck:u_0");
        multivariate_challenge.emplace_back(round_challenge);
        // Prepare sumcheck book-keeping table for the next round
        partially_evaluate(full_polynomials, multivariate_n, round_challenge);
        // Prepare ZK Sumcheck data for the next round
        if constexpr (Flavor::HasZK) {
            update_zk_sumcheck_data(zk_sumcheck_data, round_challenge, round_idx);
        };
        pow_univariate.partially_evaluate(round_challenge);
        round.round_size = round.round_size >> 1; // TODO(#224)(Cody): Maybe partially_evaluate should do this and
                                                  // release memory?        // All but final round
                                                  // We operate on partially_evaluated_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(round_idx,
                                                        partially_evaluated_polynomials,
                                                        relation_parameters,
                                                        pow_univariate,
                                                        alpha,
                                                        zk_sumcheck_data);
            // Place evaluations of Sumcheck Round Univariate in the transcript
            transcript->send_to_verifier("Sumcheck:univariate_" + std::to_string(round_idx), round_univariate);
            FF round_challenge = transcript->template get_challenge<FF>("Sumcheck:u_" + std::to_string(round_idx));
            multivariate_challenge.emplace_back(round_challenge);
            // Prepare sumcheck book-keeping table for the next round
            partially_evaluate(partially_evaluated_polynomials, round.round_size, round_challenge);
            // Prepare evaluation masking and libra structures for the next round (for ZK Flavors)
            if constexpr (Flavor::HasZK) {
                update_zk_sumcheck_data(zk_sumcheck_data, round_challenge, round_idx);
            };

            pow_univariate.partially_evaluate(round_challenge);
            round.round_size = round.round_size >> 1;
        }
        // Check that the challenges \f$ u_0,\ldots, u_{d-1} \f$ do not satisfy the equation \f$ u_0(1-u_0) + \ldots +
        // u_{d-1} (1 - u_{d-1}) = 0 \f$. This equation is satisfied with probability ~ 1/|FF|, in such cases the prover
        // has to abort and start ZK Sumcheck anew.
        if constexpr (Flavor::HasZK) {
            check_that_evals_do_not_leak_witness_data(multivariate_challenge);
        };
        // Zero univariates are used to pad the proof to the fixed size CONST_PROOF_SIZE_LOG_N.
        auto zero_univariate = bb::Univariate<FF, Flavor::BATCHED_RELATION_PARTIAL_LENGTH>::zero();
        for (size_t idx = multivariate_d; idx < CONST_PROOF_SIZE_LOG_N; idx++) {
            transcript->send_to_verifier("Sumcheck:univariate_" + std::to_string(idx), zero_univariate);
            FF round_challenge = transcript->template get_challenge<FF>("Sumcheck:u_" + std::to_string(idx));
            multivariate_challenge.emplace_back(round_challenge);
        }
        // The evaluations of Libra uninvariates at \f$ g_0(u_0), \ldots, g_{d-1} (u_{d-1}) \f$ are added to the
        // transcript.
        if constexpr (Flavor::HasZK) {
            transcript->send_to_verifier("Libra:evaluations", zk_sumcheck_data.libra_evaluations);
        };

        // Claimed evaluations of Prover polynomials are extracted and added to the transcript. When Flavor has ZK, the
        // evaluations of all witnesses are masked.
        ClaimedEvaluations multivariate_evaluations;
        multivariate_evaluations = extract_claimed_evaluations(partially_evaluated_polynomials);
        transcript->send_to_verifier("Sumcheck:evaluations", multivariate_evaluations.get_all());
        // For ZK Flavors: the evaluations of Libra univariates are included in the Sumcheck Output
        if constexpr (!Flavor::HasZK) {
            return SumcheckOutput<Flavor>{ multivariate_challenge, multivariate_evaluations };
        } else {
            return SumcheckOutput<Flavor>{ multivariate_challenge,
                                           multivariate_evaluations,
                                           zk_sumcheck_data.libra_evaluations };
        }
    };

    /**
     *
     @brief Evaluate Honk polynomials at the round challenge and prepare class for next round.
     @details At initialization, \ref ProverPolynomials "Prover Polynomials"
     are submitted by reference into \p full_polynomials, which is a two-dimensional array defined as \f{align}{
    \texttt{full_polynomials}_{i,j} = P_j(\vec i). \f} Here, \f$ \vec i \in \{0,1\}^d \f$ is identified with the binary
    representation of the integer \f$ 0 \leq i \leq 2^d-1 \f$.

     * When the first challenge \f$ u_0 \f$ is computed, the method \ref partially_evaluate "partially evaluate" takes
    as input \p full_polynomials and populates  \ref partially_evaluated_polynomials "a new book-keeping table" denoted
    \f$\texttt{partially_evaluated_polynomials}\f$. Its \f$ n/2 = 2^{d-1} \f$ rows represent the evaluations  \f$
    P_i(u_0, X_1, ..., X_{d-1}) \f$, which are multilinear polynomials in \f$ d-1 \f$ variables.
     * More precisely, it is a table  \f$ 2^{d-1} \f$ rows and \f$ N \f$ columns, such that
    \f{align}{ \texttt{partially_evaluated_polynomials}_{i,j} = &\ P_j(0, i_1,\ldots, i_{d-1}) + u_0 \cdot (P_j(1,
    i_1,\ldots, i_{d-1})) - P_j(0, i_1,\ldots, i_{d-1})) \\ = &\ \texttt{full_polynomials}_{2 i,j} + u_0 \cdot
    (\texttt{full_polynomials}_{2i+1,j} - \texttt{full_polynomials}_{2 i,j}) \f}
     * We elude copying all of the polynomial-defining data by only populating \ref partially_evaluated_polynomials
    after the first round.

     * In Round \f$0<i\leq d-1\f$, this method takes the challenge \f$ u_{i} \f$ and rewrites the first \f$ 2^{d-i-1}
    \f$ rows in the \f$ \texttt{partially_evaluated_polynomials} \f$ table with the values
     * \f{align}{
        \texttt{partially_evaluated_polynomials}_{\ell,j} \gets &\
         P_j\left(u_0,\ldots, u_{i}, \vec \ell \right)    \\
       = &\ P_j\left(u_0,\ldots, u_{i-1}, 0,  \vec \ell \right) + u_{i} \cdot \left( P_j\left(u_0, \ldots, u_{i-1}, 1,
    \vec \ell ) - P_j(u_0,\ldots, u_{i-1}, 0,  \vec \ell \right)\right)  \\ =
    &\ \texttt{partially_evaluated_polynomials}_{2 \ell,j}  + u_{i} \cdot (\texttt{partially_evaluated_polynomials}_{2
    \ell+1,j} - \texttt{partially_evaluated_polynomials}_{2\ell,j}) \f} where \f$\vec \ell \in \{0,1\}^{d-1-i}\f$.
     * After the final update, i.e. when \f$ i = d-1 \f$, the upper row of the table contains the evaluations of Honk
     * polynomials at the challenge point \f$ (u_0,\ldots, u_{d-1}) \f$.
     * @param polynomials Honk polynomials at initialization; partially evaluated polynomials in subsequent rounds
     * @param round_size \f$2^{d-i}\f$
     * @param round_challenge \f$u_i\f$
     */
    void partially_evaluate(auto& polynomials, size_t round_size, FF round_challenge)
    {
        auto pep_view = partially_evaluated_polynomials.get_all();
        auto poly_view = polynomials.get_all();
        // after the first round, operate in place on partially_evaluated_polynomials
        parallel_for(poly_view.size(), [&](size_t j) {
            for (size_t i = 0; i < round_size; i += 2) {
                pep_view[j][i >> 1] = poly_view[j][i] + round_challenge * (poly_view[j][i + 1] - poly_view[j][i]);
            }
        });
    };
    /**
     * @brief Evaluate at the round challenge and prepare class for next round.
     * Specialization for array, see \ref bb::SumcheckProver<Flavor>::partially_evaluate "generic version".
     */
    template <typename PolynomialT, std::size_t N>
    void partially_evaluate(std::array<PolynomialT, N>& polynomials, size_t round_size, FF round_challenge)
    {
        auto pep_view = partially_evaluated_polynomials.get_all();
        // after the first round, operate in place on partially_evaluated_polynomials
        parallel_for(polynomials.size(), [&](size_t j) {
            for (size_t i = 0; i < round_size; i += 2) {
                pep_view[j][i >> 1] = polynomials[j][i] + round_challenge * (polynomials[j][i + 1] - polynomials[j][i]);
            }
        });
    };

    /**
    * @brief This method takes the book-keeping table containing partially evaluated prover polynomials and creates a
    * vector containing the evaluations of all prover polynomials at the point \f$ (u_0, \ldots, u_{d-1} )\f$.
    * For ZK Flavors: this method takes the book-keeping table containing partially evaluated prover polynomials
and creates a vector containing the evaluations of all witness polynomials at the point \f$ (u_0, \ldots, u_{d-1} )\f$
masked by the terms \f$ \texttt{eval_masking_scalars}_j\cdot \sum u_i(1-u_i)\f$ and the evaluations of all non-witness
polynomials that are sent in clear.
    *
    * @param partially_evaluated_polynomials
    * @param multivariate_evaluations
    */
    ClaimedEvaluations extract_claimed_evaluations(PartiallyEvaluatedMultivariates& partially_evaluated_polynomials)
    {
        ClaimedEvaluations multivariate_evaluations;
        if constexpr (!Flavor::HasZK) {
            for (auto [eval, poly] :
                 zip_view(multivariate_evaluations.get_all(), partially_evaluated_polynomials.get_all())) {
                eval = poly[0];
            };
        } else {
            // Extract claimed evaluations of non-witness polynomials
            for (auto [eval, poly] : zip_view(multivariate_evaluations.get_non_witnesses(),
                                              partially_evaluated_polynomials.get_non_witnesses())) {
                eval = poly[0];
            };
            // Extract claimed evaluations of all witness polynomials
            for (auto [eval, poly, masking_term] : zip_view(multivariate_evaluations.get_all_witnesses(),
                                                            partially_evaluated_polynomials.get_all_witnesses(),
                                                            zk_sumcheck_data.masking_terms_evaluations)) {
                eval = poly[0] + masking_term.value_at(0);
            }
        }
        return multivariate_evaluations;
    };

    /**
     * @brief Create and populate the structure required for the ZK Sumcheck.

     * @details This method creates an array of random field elements \f$ \rho_1,\ldots, \rho_{N_w}\f$ aimed to mask the
    evaluations of witness polynomials, these are contained in \f$ \texttt{eval_masking_scalars} \f$. In order to
    optimize the computation of Sumcheck Round Univariates, it populates a table of univariates \f$
    \texttt{masking_terms_evaluations} \f$ which contains at the beginning the evaluations of polynomials \f$ \rho_j
    \cdot (1-X)\cdot X \f$ at \f$ 0,\ldots, \text{MAX_PARTIAL_RELATION_LENGTH} - 1\f$. This method also creates Libra
    univariates, computes the Libra total sum and adds it to the transcript, and sets up all auxiliary objects.
     *
     * @param zk_sumcheck_data
     */
    void setup_zk_sumcheck_data(ZKSumcheckData<Flavor>& zk_sumcheck_data)
    {

        EvalMaskingScalars eval_masking_scalars;

        for (size_t k = 0; k < NUM_ALL_WITNESS_ENTITIES; ++k) {
            eval_masking_scalars[k] = FF::random_element();
        };
        // Generate random scalars \f$ \rho_1,\ldots, \rho_{N_w}\f$ to mask the evaluations of witness polynomials and
        // populate the table masking_terms_evaluations with the terms \f$ \rho_j \cdot (1-k) \cdot k \f$
        auto masking_terms_evaluations = create_evaluation_masking_table(eval_masking_scalars);
        //  Generate random Libra Polynomials to mask Round Univariates.
        LibraUnivariates libra_univariates = generate_libra_polynomials(multivariate_d);
        // have to commit to libra_univariates here
        auto libra_scaling_factor = FF(1);
        FF libra_total_sum = compute_libra_total_sum(libra_univariates, libra_scaling_factor);
        transcript->send_to_verifier("Libra:Sum", libra_total_sum);
        // get the challenge for the zk-sumcheck claim \sigma + \rho \cdot libra_total_sum
        FF libra_challenge = transcript->template get_challenge<FF>("Libra:Challenge");
        // Initialize Libra running sum by multiplpying it by Libra challenge \f$\rho\f$;
        auto libra_running_sum = libra_total_sum * libra_challenge;
        // Multiply the column-univariates of the array of libra polynomials by libra challenge and power of \f$ 2\f$,
        // modify libra running_sum subtracting the contribution from the first univariate
        setup_libra_data(libra_univariates, libra_scaling_factor, libra_challenge, libra_running_sum);

        std::vector<FF> libra_evaluations;
        libra_evaluations.reserve(multivariate_d);
        zk_sumcheck_data = ZKSumcheckData<Flavor>(eval_masking_scalars,
                                                  masking_terms_evaluations,
                                                  libra_univariates,
                                                  libra_scaling_factor,
                                                  libra_challenge,
                                                  libra_running_sum,
                                                  libra_evaluations);
    };

    /**
     * @brief Given number of univariate polynomials and the number of their evaluations meant to be hidden, this method
     * produces a vector of univariate polynomials of degree \ref ZK_BATCHED_LENGTH "ZK_BATCHED_LENGTH - 1" with
     * independent uniformly random coefficients.
     *
     */
    static LibraUnivariates generate_libra_polynomials(size_t number_of_polynomials)
    {
        LibraUnivariates libra_full_polynomials(number_of_polynomials);
        for (auto& libra_polynomial : libra_full_polynomials) {
            // generate random polynomial of required size
            libra_polynomial = bb::Univariate<FF, LIBRA_UNIVARIATES_LENGTH>::get_random();
        };

        return libra_full_polynomials;
    };
    /**
     * @brief Generate an array of random scalars of size equal to the number of all witness polynomials and populate a
     * table of evaluations of the quadratic terms needed for masking evaluations of witnesses.
     *
     * @param evaluations
     */
    static EvaluationMaskingTable create_evaluation_masking_table(EvalMaskingScalars eval_masking_scalars)
    {
        EvaluationMaskingTable output_table;
        for (size_t column_idx = 0; column_idx < NUM_ALL_WITNESS_ENTITIES; ++column_idx) {
            for (size_t row_idx = 0; row_idx < MAX_PARTIAL_RELATION_LENGTH; ++row_idx) {
                auto scalar = FF(row_idx);
                output_table[column_idx].value_at(row_idx) =
                    scalar * (FF(1) - scalar) * eval_masking_scalars[column_idx];
            };
        };
        return output_table;
    };

    /**
     * @brief Update the table of masking quadratic terms by adding a contribution from a current challenge.
     *
     @details At initialization, \f$j\f$'th column of the masking terms evaluations table is a vector \f$(0, 0, \rho_2
     \cdot 2, \ldots, \rho_j \cdot k (1-k), \ldots, \rho_j \cdot (D-1) (1-(D-1)))\f$. Upon getting current round
     challenge, the prover adds the term \f$ \rho_j \cdot u_i \cdot (1-u_i)\f$ to each entry in the table.

     It is useful at the stage of evaluating the relation \f$ \tilde{F} \f$ at the arguments given by the values of
     \f$(\widehat{P}_1, \ldots, \widehat{P}_{N_w})\f$ at the points \f$u_0,\ldots, u_{i}, k, \vec \ell)\f$.
     * @param evaluations
     * @param masking_scalars
     * @param round_challenge
     */
    void update_masking_terms_evaluations(ZKSumcheckData<Flavor>& zk_sumcheck_data, FF round_challenge)
    {
        for (auto [masking_term, masking_scalar] :
             zip_view(zk_sumcheck_data.masking_terms_evaluations, zk_sumcheck_data.eval_masking_scalars)) {
            for (size_t k = 0; k < MAX_PARTIAL_RELATION_LENGTH; ++k) {
                masking_term.value_at(k) += round_challenge * (FF(1) - round_challenge) * masking_scalar;
            }
        }
    }
    /**
     * @brief Compute the sum of the randomly sampled multivariate polynomial \f$ G = \sum_{i=0}^{n-1} g_i(X_i) \f$ over
     * the Boolean hypercube.
     *
     * @param libra_univariates
     * @param scaling_factor
     * @return FF
     */
    static FF compute_libra_total_sum(auto libra_univariates, FF& scaling_factor)
    {
        FF total_sum = 0;
        scaling_factor = scaling_factor / 2;

        for (auto univariate : libra_univariates) {
            total_sum += univariate.value_at(0) + univariate.value_at(1);
            scaling_factor *= 2;
        }
        total_sum *= scaling_factor;

        return total_sum;
    }
    /**
     * @brief Set up Libra book-keeping table that simplifies the computation of Libra Round Univariates
     *
     * @details The array of Libra univariates is getting scaled
     * \f{align}{
        \texttt{libra_univariates} \gets \texttt{libra_univariates}\cdot \rho \cdot 2^{d-1}
     \f}
     * We also initialize
     * \f{align}{
            \texttt{libra_running_sum} \gets \texttt{libra_total_sum} - \texttt{libra_univariates}_{0,0} -
     \texttt{libra_univariates}_{0,1} \f}.
     * @param libra_table
     * @param libra_round_factor
     * @param libra_challenge
     */
    void setup_libra_data(auto& libra_univariates,
                          FF& libra_scaling_factor,
                          const FF libra_challenge,
                          FF& libra_running_sum)
    {
        libra_scaling_factor *= libra_challenge; // \rho * 2^{d-1}
        for (auto& univariate : libra_univariates) {
            univariate *= libra_scaling_factor;
        };
        // subtract the contribution of the first libra univariate from libra total sum
        libra_running_sum += -libra_univariates[0].value_at(0) - libra_univariates[0].value_at(1);
        libra_running_sum *= FF(1) / FF(2);
    }

    /**
     * @brief Upon receiving the challenge \f$u_i\f$, the prover updates Libra data. If \f$ i < d-1\f$

        -  update the table of Libra univariates by multiplying every term by \f$1/2\f$.
        -  computes the value \f$2^{d-i - 2} \cdot \texttt{libra_challenge} \cdot g_0(u_0)\f$ applying \ref
        bb::Univariate::evaluate "evaluate" method to the first univariate in the table \f$\texttt{libra_univariates}\f$
        -  places the value \f$ g_0(u_0)\f$ to the vector \f$ \texttt{libra_evaluations}\f$
        -  update the running sum
        \f{align}{
                \texttt{libra_running_sum} \gets  2^{d-i-2} \cdot \texttt{libra_challenge} \cdot g_0(u_0) +  2^{-1}
     \cdot \left( \texttt{libra_running_sum} - (\texttt{libra_univariates}_{i+1}(0) +
     \texttt{libra_univariates}_{i+1}(1)) \right) \f} If \f$ i = d-1\f$
        -  compute the value \f$ g_{d-1}(u_{d-1})\f$ applying \ref bb::Univariate::evaluate "evaluate" method to the
     last univariate in the table \f$\texttt{libra_univariates}\f$ and dividing the result by \f$
     \texttt{libra_challenge} \f$.
        -  update the table of Libra univariates by multiplying every term by \f$\texttt{libra_challenge}^{-1}\f$.
     @todo Refactor once the Libra univariates are extracted from the Proving Key. Then the prover does not need to
        update the first round_idx - 1 univariates and could release the memory. Also, use batch_invert / reduce
        the number of divisions by 2.
     * @param libra_univariates
     * @param round_challenge
     * @param round_idx
     * @param libra_running_sum
     * @param libra_evaluations
     */
    void update_libra_data(ZKSumcheckData<Flavor>& zk_sumcheck_data, const FF round_challenge, size_t round_idx)
    {
        // when round_idx = d - 1, the update is not needed
        if (round_idx < zk_sumcheck_data.libra_univariates.size() - 1) {
            for (auto& univariate : zk_sumcheck_data.libra_univariates) {
                univariate *= FF(1) / FF(2);
            };
            // compute the evaluation \f$ \rho \cdot 2^{d-2-i} \çdot g_i(u_i) \f$
            auto libra_evaluation = zk_sumcheck_data.libra_univariates[round_idx].evaluate(round_challenge);
            auto next_libra_univariate = zk_sumcheck_data.libra_univariates[round_idx + 1];
            // update the running sum by adding g_i(u_i) and subtracting (g_i(0) + g_i(1))
            zk_sumcheck_data.libra_running_sum +=
                -next_libra_univariate.value_at(0) - next_libra_univariate.value_at(1);
            zk_sumcheck_data.libra_running_sum *= FF(1) / FF(2);

            zk_sumcheck_data.libra_running_sum += libra_evaluation;
            zk_sumcheck_data.libra_scaling_factor *= FF(1) / FF(2);

            zk_sumcheck_data.libra_evaluations.emplace_back(libra_evaluation / zk_sumcheck_data.libra_scaling_factor);
        } else {
            // compute the evaluation of the last Libra univariate at the challenge u_{d-1}
            auto libra_evaluation = zk_sumcheck_data.libra_univariates[round_idx].evaluate(round_challenge) /
                                    zk_sumcheck_data.libra_scaling_factor;
            // place the evalution into the vector of Libra evaluations
            zk_sumcheck_data.libra_evaluations.emplace_back(libra_evaluation);
            for (auto univariate : zk_sumcheck_data.libra_univariates) {
                univariate *= FF(1) / zk_sumcheck_data.libra_challenge;
            }
        };
    }

    void update_zk_sumcheck_data(ZKSumcheckData<Flavor>& zk_sumcheck_data, FF round_challenge, size_t round_idx)
    {
        update_libra_data(zk_sumcheck_data, round_challenge, round_idx);
        update_masking_terms_evaluations(zk_sumcheck_data, round_challenge);
    }
    /**
     * @brief By the design of ZK Sumcheck, instead of claimed evaluations of witness polynomials \f$ P_1, \ldots,
    P_{N_w} \f$, the prover sends the evaluations of the witness polynomials masked by the terms \f$ \rho_j
    \sum_{i=0}^{d-1} u_i(1-u_i) \f$ for \f$ j= 1, \ldots N_w\f$. If the challenges satisfy the equation
    \f$\sum_{i=0}^{d-1} u_i(1-u_i) = 0\f$, each masking term is \f$0 \f$, which could lead to the leakage of witness
     *
     * @param multivariate_challenge
     */
    void check_that_evals_do_not_leak_witness_data(std::vector<FF> multivariate_challenge)
    {
        auto masking_term = FF(0);
        for (auto challenge : multivariate_challenge) {
            masking_term += challenge * (FF(1) - challenge);
        }
        if (masking_term == FF(0)) {
            throw_or_abort("The evaluations of witness polynomials are not masked, because u_0(1-u_0)+...+u_{d-1} "
                           "(1-u_{d-1}) = 0 ");
        };
    }
};
/*! \brief Implementation of the sumcheck Verifier for statements of the form \f$\sum_{\vec \ell \in \{0,1\}^d}
 pow_{\beta}(\vec \ell) \cdot F \left(P_1(\vec \ell),\ldots, P_N(\vec \ell) \right)  = 0 \f$ for multilinear
 polynomials \f$P_1, \ldots, P_N \f$.
 *
  \class SumcheckVerifier
  \details
 * Init:
 * - Claimed Sumcheck sum: \f$\quad \sigma_{ 0 } \gets 0 \f$
 *
 * For \f$ i = 0,\ldots, d-1\f$:
 * - Extract Round Univariate's \f$\tilde{F}\f$ evaluations at \f$0,\ldots, D \f$ from the transcript using \ref
 bb::BaseTranscript::receive_from_prover "receive_from_prover" method from \ref bb::BaseTranscript< TranscriptParams >
 "Base Transcript Class".
 * - \ref bb::SumcheckVerifierRound< Flavor >::check_sum "Check target sum": \f$\quad \sigma_{
 i } \stackrel{?}{=}  \tilde{S}^i(0) + \tilde{S}^i(1)  \f$
 * - Compute the challenge \f$u_i\f$ from the transcript using \ref bb::BaseTranscript::get_challenge "get_challenge"
 method.
 * - \ref bb::SumcheckVerifierRound< Flavor >::compute_next_target_sum "Compute next target sum" :\f$ \quad \sigma_{i+1}
 \gets \tilde{S}^i(u_i) \f$
 * ### Verifier's Data before Final Step
 * Entering the final round, the Verifier has already checked that \f$\quad \sigma_{ d-1 } = \tilde{S}^{d-2}(u_{d-2})
 \stackrel{?}{=}  \tilde{S}^{d-1}(0) + \tilde{S}^{d-1}(1)  \f$ and computed \f$\sigma_d = \tilde{S}^{d-1}(u_{d-1})\f$.
 * ### Final Verification Step
 * - Extract \ref ClaimedEvaluations of prover polynomials \f$P_1,\ldots, P_N\f$ at the challenge point \f$
 (u_0,\ldots,u_{d-1}) \f$ from the transcript and \ref bb::SumcheckVerifierRound< Flavor
 >::compute_full_honk_relation_purported_value "compute evaluation:"
 \f{align}{\tilde{F}\left( P_1(u_0,\ldots, u_{d-1}), \ldots, P_N(u_0,\ldots, u_{d-1}) \right)\f}
 and store it at \f$ \texttt{full_honk_relation_purported_value} \f$.
 * - Compare \f$ \sigma_d \f$ against the evaluation of \f$ \tilde{F} \f$ at \f$P_1(u_0,\ldots, u_{d-1}), \ldots,
 P_N(u_0,\ldots, u_{d-1})\f$:
 * \f{align}{\quad  \sigma_{ d } \stackrel{?}{=} \tilde{F}\left(P_1(u_{0}, \ldots, u_{d-1}),\ldots, P_N(u_0,\ldots,
 u_{d-1})\right)\f}

  \snippet cpp/src/barretenberg/sumcheck/sumcheck.hpp Final Verification Step

 */
template <typename Flavor> class SumcheckVerifier {

  public:
    using Utils = bb::RelationUtils<Flavor>;
    using FF = typename Flavor::FF;
    /**
     * @brief Container type for the evaluations of Prover Polynomials \f$P_1,\ldots,P_N\f$ at the challenge point
     * \f$(u_0,\ldots, u_{d-1}) \f$.
     *
     */
    using ClaimedEvaluations = typename Flavor::AllValues;
    // For ZK Flavors: the verifier obtains a vector of evaluations of \f$ d \f$ univariate polynomials and uses them to
    // compute full_honk_relation_purported_value
    using ClaimedLibraEvaluations = typename std::vector<FF>;
    using Transcript = typename Flavor::Transcript;
    using RelationSeparator = typename Flavor::RelationSeparator;

    /**
     * @brief Maximum partial algebraic degree of the relation  \f$\tilde F = pow_{\beta} \cdot F \f$, i.e. \ref
     * MAX_PARTIAL_RELATION_LENGTH "MAX_PARTIAL_RELATION_LENGTH + 1".
     */
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
    /**
     * @brief The number of Prover Polynomials \f$ P_1, \ldots, P_N \f$ specified by the Flavor.
     *
     */
    static constexpr size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    /**
     * @brief Number of variables in Prover Polynomials.
     *
     */
    const size_t multivariate_d;

    std::shared_ptr<Transcript> transcript;
    SumcheckVerifierRound<Flavor> round;

    // Verifier instantiates sumcheck with circuit size, optionally a different target sum than 0 can be specified.
    explicit SumcheckVerifier(size_t multivariate_d, std::shared_ptr<Transcript> transcript, FF target_sum = 0)
        : multivariate_d(multivariate_d)
        , transcript(transcript)
        , round(target_sum){};
    /**
     * @brief Extract round univariate, check sum, generate challenge, compute next target sum..., repeat until
     * final round, then use purported evaluations to generate purported full Honk relation value and check against
     * final target sum.
     *
     * @details If verification fails, returns std::nullopt, otherwise returns SumcheckOutput
     * @param relation_parameters
     * @param transcript
     */
    SumcheckOutput<Flavor> verify(const bb::RelationParameters<FF>& relation_parameters,
                                  RelationSeparator alpha,
                                  const std::vector<FF>& gate_challenges)
    {
        bool verified(true);

        bb::PowPolynomial<FF> pow_univariate(gate_challenges);
        // All but final round.
        // target_total_sum is initialized to zero then mutated in place.

        if (multivariate_d == 0) {
            throw_or_abort("Number of variables in multivariate is 0.");
        }

        FF libra_challenge;
        FF libra_total_sum;
        if constexpr (Flavor::HasZK) {
            // get the claimed sum of libra masking multivariate over the hypercube
            libra_total_sum = transcript->template receive_from_prover<FF>("Libra:Sum");
            // get the challenge for the ZK Sumcheck claim
            libra_challenge = transcript->template get_challenge<FF>("Libra:Challenge");
        }
        std::vector<FF> multivariate_challenge;
        multivariate_challenge.reserve(multivariate_d);
        // if Flavor has ZK, the target total sum is corrected by Libra total sum multiplied by the Libra
        // challenge
        if constexpr (Flavor::HasZK) {
            round.target_total_sum += libra_total_sum * libra_challenge;
        };
        for (size_t round_idx = 0; round_idx < CONST_PROOF_SIZE_LOG_N; round_idx++) {
            // Obtain the round univariate from the transcript
            std::string round_univariate_label = "Sumcheck:univariate_" + std::to_string(round_idx);
            auto round_univariate =
                transcript->template receive_from_prover<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                    round_univariate_label);
            FF round_challenge = transcript->template get_challenge<FF>("Sumcheck:u_" + std::to_string(round_idx));

            if constexpr (IsRecursiveFlavor<Flavor>) {
                typename Flavor::CircuitBuilder* builder = round_challenge.get_context();
                stdlib::bool_t dummy_round = stdlib::witness_t(builder, round_idx >= multivariate_d);
                bool checked = round.check_sum(round_univariate, dummy_round);
                // Only utilize the checked value if this is not a constant proof size padding round
                if (round_idx < multivariate_d) {
                    verified = verified && checked;
                }
                multivariate_challenge.emplace_back(round_challenge);

                round.compute_next_target_sum(round_univariate, round_challenge, dummy_round);
                pow_univariate.partially_evaluate(round_challenge, dummy_round);

            } else {
                if (round_idx < multivariate_d) {
                    bool checked = round.check_sum(round_univariate);
                    verified = verified && checked;
                    multivariate_challenge.emplace_back(round_challenge);
                    round.compute_next_target_sum(round_univariate, round_challenge);
                    pow_univariate.partially_evaluate(round_challenge);
                } else {
                    multivariate_challenge.emplace_back(round_challenge);
                }
            }
        }
        // Extract claimed evaluations of Libra univariates and compute their sum multiplied by the Libra challenge
        ClaimedLibraEvaluations libra_evaluations(multivariate_d);
        FF full_libra_purported_value = FF(0);
        if constexpr (Flavor::HasZK) {
            for (size_t idx = 0; idx < multivariate_d; idx++) {
                libra_evaluations[idx] =
                    transcript->template receive_from_prover<FF>("libra_evaluation" + std::to_string(idx));
                full_libra_purported_value += libra_evaluations[idx];
            };
            full_libra_purported_value *= libra_challenge;
        };
        // Final round
        ClaimedEvaluations purported_evaluations;
        auto transcript_evaluations =
            transcript->template receive_from_prover<std::array<FF, NUM_POLYNOMIALS>>("Sumcheck:evaluations");
        for (auto [eval, transcript_eval] : zip_view(purported_evaluations.get_all(), transcript_evaluations)) {
            eval = transcript_eval;
        }
        // Evaluate the Honk relation at the point (u_0, ..., u_{d-1}) using claimed evaluations of prover polynomials.
        // In ZK Flavors, the evaluation is corrected by full_libra_purported_value
        FF full_honk_purported_value = round.compute_full_honk_relation_purported_value(
            purported_evaluations, relation_parameters, pow_univariate, alpha, full_libra_purported_value);
        bool final_check(false);
        //! [Final Verification Step]
        if constexpr (IsRecursiveFlavor<Flavor>) {
            final_check = (full_honk_purported_value.get_value() == round.target_total_sum.get_value());
        } else {
            final_check = (full_honk_purported_value == round.target_total_sum);
        }
        verified = final_check && verified;
        // For ZK Flavors: the evaluations of Libra univariates are included in the Sumcheck Output
        if constexpr (!Flavor::HasZK) {
            return SumcheckOutput<Flavor>{ multivariate_challenge, purported_evaluations, verified };
        } else {
            return SumcheckOutput<Flavor>{ multivariate_challenge, purported_evaluations, libra_evaluations, verified };
        }
    };
};
} // namespace bb
