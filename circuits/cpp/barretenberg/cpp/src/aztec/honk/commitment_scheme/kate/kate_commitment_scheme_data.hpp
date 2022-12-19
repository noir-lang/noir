#pragma once

#include <srs/reference_string/reference_string.hpp>
#include "../commitment_scheme.hpp"

/**
 * @brief describes the data structures used by the Kate commitment scheme. Conforms to the CommitmentSchemeData
 * specification (TODO make into a concept)
 *
 * @tparam Curve the elliptic curve being used
 *
 * @details The purpose of this class is to provide an abstraction layer that prevents Kate-specific data types from
 * leaking out of our CommitmentScheme concept. For example, consider a proof system that has a CommitmentScheme
 * template parameter. If parametrised using Kate, commitments will be Curve::G1::affine_element types. If parametrised
 * using IPA, commitments will be std::vector<Curve::G1::affine_element> types For the proof system to be
 * commitment-scheme agnostic we need a single unifying type for commitments from any commitment scheme This is
 * CommitmentSchemeData exposes a generic `Commitment` type Now the above proof system can use
 * `CommitmentScheme::Commitment` to refer to its commitments and not have to worry about underlying type details
 */
template <typename Curve> class KateCommitmentSchemeData {
  public:
    // expose field and group elements for convenience
    typedef typename Curve::Fq Fq;
    typedef typename Curve::Fr Fr;
    typedef typename Curve::G1 G1;

    // define how Kate represents commitments
    typedef typename G1::affine_element Commitment;

    // expose a generic type for the ProverSRS and VerifierSRS as this may differ depending on commitment scheme
    typedef waffle::ProverReferenceString SRS;
    typedef waffle::VerifierReferenceString VerifierSRS;

    // TODO: this only works or Fr?! We need to refactor `polynomial` to be field-agnostic
    typedef barretenberg::polynomial Polynomial;

    // // NOTE: this sort of aliasing is not allowed by gcc!
    // using OpeningSchema = OpeningSchema<Fr, Commitment>;
    // using OpeningProof = OpeningProof<Fr, Commitment>;

    // struct OpeningSchema {
    //     std::vector<Fr*> polynomials;
    //     std::vector<Fr*> shifted_polynomials;
    //     std::vector<Commitment> commitments;
    //     // 2d vector. First vector = number of variables, second vector = number of point openings for each variable.
    //     I
    //     // think we can assume we open all polynomials at all points to keep things simplish...
    //     std::vector<std::vector<Fr>> variables;

    //     // 2d vector. First vector = evaluations for a specific polynomial. Second vector = evaluations for a
    //     specific
    //     // polynomial given a specific opening point
    //     std::vector<std::vector<Fr>> evaluations;
    //     std::vector<std::vector<Fr>> shifted_evaluations;

    //     const size_t n;
    // };

    // /**
    //  * @brief describes information (minus challenges) required to verify a Kate opening proof.
    //  * Conforms to the OpeningProof concept
    //  *
    //  * @param commitments Commitments to the polynomials being opened
    //  * @param variables The set of points that the polynomials are being opened at
    //  * @param evaluations The claimed evaluations of the committed polynomials when evaluated at points described by
    //  * `variables`
    //  * @param PI the opening proof group elements. One for every variable
    //  */
    // struct OpeningProof {
    //     std::vector<Commitment> commitments;
    //     // variables we're opening our polynomials at, and their opening proofs
    //     std::vector<Fr> variables;
    //     // 2d vector. First vector = evaluations for a specific polynomial. Second vector = evaluations for a
    //     specific
    //     // polynomial given a specific opening point
    //     std::vector<std::vector<Fr>> evaluations;
    //     std::vector<Commitment> PI;
    // };
};
