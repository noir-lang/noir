#pragma once
#include <vector>

// TODO: this needs to be a C++20 concept!
// i.e. we don't want to inherit from CommitmentScheme, but rather this thing defines an interface
// that we want multiple classes to conform

/**
 * @brief describes information (minus challenges) required to create a Kate opening proof.
 * Conforms to the OpeningSchema concept
 *
 * @param polynomials Polynomials being opened
 * @param shifted_polynomials Shifted polynomials being opened. Should be size ZERO for Kate as shifts are
 * represented by modifying the evaluation points
 * @param commitments Commitments to the polynomials being opened
 * @param variables The set of points that the polynomials are being opened at
 * @param evaluations The claimed evaluations of the committed polynomials when evaluated at points described by
 * `variables`
 * @param evaluations The claimed evaluations of the shifted polynomials. Should be size ZERO for Kate as shfits are
 * represented by modifying the evaluation points
 * @param n the number of coefficients in our polynomials (assumed all polynomials are of size n)
 */
template <typename Fr, typename Commitment> struct OpeningSchema {
    std::vector<Fr*> polynomials;
    std::vector<Fr*> shifted_polynomials;
    std::vector<Commitment> commitments;
    // 2d vector. First vector = number of variables, second vector = number of point openings for each variable. I
    // think we can assume we open all polynomials at all points to keep things simplish...
    std::vector<std::vector<Fr>> variables;

    // 2d vector. First vector = evaluations for a specific polynomial. Second vector = evaluations for a specific
    // polynomial given a specific opening point
    std::vector<std::vector<Fr>> evaluations;
    std::vector<std::vector<Fr>> shifted_evaluations;

    const size_t n;
};

/**
 * @brief describes information (minus challenges) required to verify a Kate opening proof.
 * Conforms to the OpeningProof concept
 *
 * @param commitments Commitments to the polynomials being opened
 * @param variables The set of points that the polynomials are being opened at
 * @param evaluations The claimed evaluations of the committed polynomials when evaluated at points described by
 * `variables`
 * @param PI the opening proof group elements. One for every variable
 */
template <typename Fr, typename Commitment> struct OpeningProof {
    std::vector<Commitment> commitments;
    // variables we're opening our polynomials at, and their opening proofs
    std::vector<Fr> variables;
    // 2d vector. First vector = evaluations for a specific polynomial. Second vector = evaluations for a specific
    // polynomial given a specific opening point
    std::vector<std::vector<Fr>> evaluations;
    std::vector<Commitment> PI;
};

// namespace commitment_scheme_types
// template <typename CommitmentSchemeData> class CommitmentScheme {
//   public:
//     typedef typename CommitmentSchemeData::Fr Fr;
//     typedef typename CommitmentSchemeData::Commitment Commitment;
//     typedef typename CommitmentSchemeData::SRS SRS;
//     typedef typename CommitmentSchemeData::OpeningProof OpeningProof;

//     using ChallengeGenerator = Fr (*)(const std::vector<Fr>&);

//     static std::vector<Commitment> commit(const std::vector<Fr*>& , std::shared_ptr<SRS> const&) = 0;

//     struct OpeningSchema {
//         std::vector<Fr*> polynomials;
//         std::vector<Fr*> shifted_polynomials;
//         std::vector<Commitment> commitments;
//         // 2d vector. First vector = number of variables, second vector = number of point openings for each variable.

//         // think we can assume we open all polynomials at all points to keep things simplish...
//         std::vector<std::vector<Fr>> variables;

//         // 2d vector. First vector = evaluations for a specific polynomial. Second vector = evaluations for a
//         specific
//         // polynomial given a specific opening point
//         std::vector<std::vector<Fr>> evaluations;
//         std::vector<std::vector<Fr>> shifted_evaluations;

//         const size_t n;
//     };

//     static virtual OpeningProof batch_open(const OpeningSchema& committed_polynomials,
//                                            std::shared_ptr<SRS> const& srs,
//                                            ChallengeGenerator& challenge_generator);

//     static virtual bool batch_verify(const OpeningProof& opening_proof,
//                                      std::shared_ptr<SRS> const& srs,
//                                      ChallengeGenerator& challenge_generator);

//     // virtual AggregatedProof aggregate(const OpeningProof& opening_proof,
//     //                                   const AggregatedProof& input_aggregation,
//     //                                   std::shared_ptr<SRS> const& srs,
//     //                                   ChallengeGenerator& challenge_generator);
// };