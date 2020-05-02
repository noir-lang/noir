#pragma once
#include "../../transcript/transcript_wrappers.hpp"
#include "../types/program_witness.hpp"
#include "../verification_key/verification_key.hpp"
#include "../prover/work_queue.hpp"
#include <ecc/curves/bn254/fr.hpp>

namespace transcript {
class Transcript;
}

namespace waffle {

struct proving_key;

class ReferenceString;

class VerifierBaseWidget {
  public:
    template <typename Field> struct challenge_coefficients {
        Field alpha_base;
        Field alpha_step;
        size_t nu_index;
        size_t linear_nu_index;
    };
    VerifierBaseWidget() = default;
    VerifierBaseWidget(const VerifierBaseWidget& other) = default;

    VerifierBaseWidget(VerifierBaseWidget&& other) = default;
    virtual ~VerifierBaseWidget() = default;

    // virtual challenge_coefficients append_scalar_multiplication_inputs(
    //     verification_key*,
    //     const challenge_coefficients& challenge,
    //     const transcript::Transcript& transcript,
    //     std::vector<barretenberg::g1::affine_element>& points,
    //     std::vector<barretenberg::fr>& scalars) = 0;

    // virtual barretenberg::fr compute_batch_evaluation_contribution(verification_key*,
    //                                                                barretenberg::fr& batch_eval,
    //                                                                const barretenberg::fr& nu_base,
    //                                                                const transcript::Transcript& transcript) = 0;

    // virtual barretenberg::fr compute_quotient_evaluation_contribution(verification_key*,
    //                                                                   const barretenberg::fr& alpha_base,
    //                                                                   const transcript::Transcript&,
    //                                                                   barretenberg::fr&)
    // {
    //     return alpha_base;
    // }

    bool verify_instance_commitments()
    {
        bool valid = true;
        // TODO: if instance commitments are points at infinity, this is probably ok?
        // because selector polynomials can be all zero :/. TODO: check?
        // for (size_t i = 0; i < instance.size(); ++i)
        // {
        //     valid = valid && barretenberg::instance[i].on_curve();
        // }
        return valid;
    }
};

class ProverBaseWidget {
  protected:
    typedef barretenberg::fr fr;
    typedef barretenberg::polynomial polynomial;

  public:
    ProverBaseWidget(proving_key* input_key, program_witness* input_witness)
        : key(input_key)
        , witness(input_witness)
    {}
    ProverBaseWidget(const ProverBaseWidget& other)
        : key(other.key)
        , witness(other.witness)
    {}
    ProverBaseWidget(ProverBaseWidget&& other)
        : key(other.key)
        , witness(other.witness)
    {}

    ProverBaseWidget& operator=(const ProverBaseWidget& other)
    {
        key = other.key;
        witness = other.witness;
        return *this;
    }

    ProverBaseWidget& operator=(ProverBaseWidget&& other)
    {
        key = other.key;
        witness = other.witness;
        return *this;
    }

    virtual ~ProverBaseWidget() {}

    virtual void compute_round_commitments(transcript::Transcript&, const size_t, work_queue&){};

    virtual barretenberg::fr compute_quotient_contribution(const barretenberg::fr& alpha_base,
                                                           const transcript::Transcript& transcript) = 0;
    virtual barretenberg::fr compute_linear_contribution(const barretenberg::fr& alpha_base,
                                                         const transcript::Transcript& transcript,
                                                         barretenberg::polynomial& r) = 0;
    virtual void compute_opening_poly_contribution(const transcript::Transcript& transcript,
                                                   const bool use_linearisation) = 0;
    virtual void compute_transcript_elements(transcript::Transcript&, const bool) = 0;

    proving_key* key;
    program_witness* witness;
};

} // namespace waffle
