use crate::file_writer::BBFiles;
use crate::utils::{map_with_newline, snake_case};

pub trait ProverBuilder {
    fn create_prover_hpp(&mut self, name: &str);

    fn create_prover_cpp(&mut self, name: &str, lookup_names: &[String]);
}

impl ProverBuilder for BBFiles {
    fn create_prover_hpp(&mut self, name: &str) {
        let include_str = includes_hpp(&snake_case(name));

        let prover_hpp = format!("
    {include_str} 
    namespace bb {{
    
    class {name}Prover {{
    
        using Flavor = {name}Flavor;
        using FF = Flavor::FF;
        using PCS = Flavor::PCS;
        using Curve = Flavor::Curve;
        using ZeroMorph = ZeroMorphProver_<Curve>;
        using PCSCommitmentKey = Flavor::CommitmentKey;
        using ProvingKey = Flavor::ProvingKey;
        using Polynomial = Flavor::Polynomial;
        using ProverPolynomials = Flavor::ProverPolynomials;
        using CommitmentLabels = Flavor::CommitmentLabels;
        using Transcript = Flavor::Transcript;
    
      public:
        explicit {name}Prover(std::shared_ptr<ProvingKey> input_key, std::shared_ptr<PCSCommitmentKey> commitment_key);
    
        void execute_preamble_round();
        void execute_wire_commitments_round();
        void execute_log_derivative_inverse_round();
        void execute_relation_check_rounds();
        void execute_pcs_rounds();
    
        HonkProof export_proof();
        HonkProof construct_proof();
    
        std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();
    
        std::vector<FF> public_inputs;
    
        bb::RelationParameters<FF> relation_parameters;
    
        std::shared_ptr<ProvingKey> key;
    
        // Container for spans of all polynomials required by the prover (i.e. all multivariates evaluated by Sumcheck).
        ProverPolynomials prover_polynomials;
    
        CommitmentLabels commitment_labels;
        typename Flavor::WitnessCommitments witness_commitments;

        Polynomial quotient_W;
    
        SumcheckOutput<Flavor> sumcheck_output;
    
        std::shared_ptr<PCSCommitmentKey> commitment_key;
    
      private:
        HonkProof proof;
    }};
    
    }} // namespace bb
     
    ");
        self.write_file(
            &self.prover,
            &format!("{}_prover.hpp", snake_case(name)),
            &prover_hpp,
        );
    }

    /// Create the prover cpp file
    ///
    /// Committed polys are included as we manually unroll all commitments, as we do not commit to everything
    fn create_prover_cpp(&mut self, name: &str, lookup_names: &[String]) {
        let include_str = includes_cpp(&snake_case(name));

        let polynomial_commitment_phase = create_commitments_phase();

        let (call_log_derivative_phase, log_derivative_inverse_phase): (String, String) =
            if lookup_names.is_empty() {
                ("".to_owned(), "".to_owned())
            } else {
                (
                    "execute_log_derivative_inverse_round();".to_owned(),
                    create_log_derivative_inverse_round(lookup_names),
                )
            };

        let prover_cpp = format!("
    {include_str}
    
    namespace bb {{

    using Flavor = {name}Flavor;
    using FF = Flavor::FF;
    
    /**
     * Create {name}Prover from proving key, witness and manifest.
     *
     * @param input_key Proving key.
     * @param input_manifest Input manifest
     *
     * @tparam settings Settings class.
     * */
    {name}Prover::{name}Prover(std::shared_ptr<Flavor::ProvingKey> input_key,
                                       std::shared_ptr<PCSCommitmentKey> commitment_key)
        : key(input_key)
        , commitment_key(commitment_key)
    {{
        for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {{
            ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
                   bb::flavor_get_label(*key, key_poly));
            prover_poly = key_poly.share();
        }}
        for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {{
            ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
                   bb::flavor_get_label(*key, key_poly) + \"_shift\");
            prover_poly = key_poly.shifted();
        }}
    }}
    

    /**
     * @brief Add circuit size, public input size, and public inputs to transcript
     *
     */
    void {name}Prover::execute_preamble_round()
    {{
        const auto circuit_size = static_cast<uint32_t>(key->circuit_size);
    
        transcript->send_to_verifier(\"circuit_size\", circuit_size);
    }}
    
    /**
     * @brief Compute commitments to all of the witness wires (apart from the logderivative inverse wires)
     *
     */
    void {name}Prover::execute_wire_commitments_round()
    {{

        {polynomial_commitment_phase}

    }}

    void {name}Prover::execute_log_derivative_inverse_round()
    {{

        {log_derivative_inverse_phase}
    }}
    
    /**
     * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
     *
     */
    void {name}Prover::execute_relation_check_rounds()
    {{
        using Sumcheck = SumcheckProver<Flavor>;
    
        auto sumcheck = Sumcheck(key->circuit_size, transcript);

        FF alpha = transcript->template get_challenge<FF>(\"Sumcheck:alpha\");
        std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));

        for (size_t idx = 0; idx < gate_challenges.size(); idx++) {{
            gate_challenges[idx] = transcript->template get_challenge<FF>(\"Sumcheck:gate_challenge_\" + std::to_string(idx));
        }}
        sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha, gate_challenges);
    }}


    /**
     * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
     * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
     *
     * */
     void {name}Prover::execute_pcs_rounds()
    {{
        auto prover_opening_claim = ZeroMorph::prove(key->circuit_size,
                                                     prover_polynomials.get_unshifted(),
                                                     prover_polynomials.get_to_be_shifted(),
                                                     sumcheck_output.claimed_evaluations.get_unshifted(),
                                                     sumcheck_output.claimed_evaluations.get_shifted(),
                                                     sumcheck_output.challenge,
                                                     commitment_key,
                                                     transcript);
        PCS::compute_opening_proof(commitment_key, prover_opening_claim, transcript);
    }}

    
    HonkProof {name}Prover::export_proof()
    {{
        proof = transcript->proof_data;
        return proof;
    }}
    
    HonkProof {name}Prover::construct_proof()
    {{
        // Add circuit size public input size and public inputs to transcript.
        execute_preamble_round();
    
        // Compute wire commitments
        execute_wire_commitments_round();
    
        // Compute sorted list accumulator and commitment
        {call_log_derivative_phase}
    
        // Fiat-Shamir: alpha
        // Run sumcheck subprotocol.
        execute_relation_check_rounds();
    
        // Fiat-Shamir: rho, y, x, z
        // Execute Zeromorph multilinear PCS
        execute_pcs_rounds();
    
        return export_proof();
    }}
    
    }} // namespace bb
     
    
    ");

        self.write_file(
            &self.prover,
            &format!("{}_prover.cpp", snake_case(name)),
            &prover_cpp,
        );
    }
}

fn includes_hpp(name: &str) -> String {
    format!(
        "
#pragma once
#include \"barretenberg/commitment_schemes/zeromorph/zeromorph.hpp\"
#include \"barretenberg/plonk/proof_system/types/proof.hpp\"
#include \"barretenberg/relations/relation_parameters.hpp\"
#include \"barretenberg/sumcheck/sumcheck_output.hpp\"
#include \"barretenberg/transcript/transcript.hpp\"

#include \"barretenberg/vm/generated/{name}_flavor.hpp\"

    "
    )
}

fn includes_cpp(name: &str) -> String {
    format!(
        "
    
    #include \"{name}_prover.hpp\"
    #include \"barretenberg/commitment_schemes/claim.hpp\"
    #include \"barretenberg/commitment_schemes/commitment_key.hpp\"
    #include \"barretenberg/honk/proof_system/logderivative_library.hpp\"
    #include \"barretenberg/honk/proof_system/permutation_library.hpp\"
    #include \"barretenberg/plonk_honk_shared/library/grand_product_library.hpp\"
    #include \"barretenberg/polynomials/polynomial.hpp\"
    #include \"barretenberg/relations/permutation_relation.hpp\"
    #include \"barretenberg/sumcheck/sumcheck.hpp\"
    "
    )
}

/// Commitment Transform
///
/// Produces code to perform kzg commitment, then stores in the witness_commitments struct
fn commitment_transform(name: &String) -> String {
    format!("witness_commitments.{name} = commitment_key->commit(key->{name});")
}

/// Send to Verifier Transform
///
/// Sends commitment produces in commitment_transform to the verifier
fn send_to_verifier_transform(name: &String) -> String {
    format!("transcript->send_to_verifier(commitment_labels.{name}, witness_commitments.{name});")
}

fn create_commitments_phase() -> String {
    format!(
        "
        // Commit to all polynomials (apart from logderivative inverse polynomials, which are committed to in the later logderivative phase)
        auto wire_polys = prover_polynomials.get_wires();
        auto labels = commitment_labels.get_wires();
        for (size_t idx = 0; idx < wire_polys.size(); ++idx) {{
            transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
        }}
        "
    )
}

fn create_log_derivative_inverse_round(lookup_operations: &[String]) -> String {
    let all_commit_operations = map_with_newline(lookup_operations, commitment_transform);
    let send_to_verifier_operations =
        map_with_newline(lookup_operations, send_to_verifier_transform);

    format!(
        "
        auto [beta, gamm] = transcript->template get_challenges<FF>(\"beta\", \"gamma\");
        relation_parameters.beta = beta;
        relation_parameters.gamma = gamm;

        key->compute_logderivative_inverses(relation_parameters);

        // Commit to all logderivative inverse polynomials
        {all_commit_operations}

        // Send all commitments to the verifier
        {send_to_verifier_operations}
        "
    )
}
