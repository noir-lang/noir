use crate::{
    file_writer::BBFiles,
    utils::{map_with_newline, snake_case},
};

pub trait VerifierBuilder {
    fn create_verifier_cpp(
        &mut self,
        name: &str,
        witness: &[String],
        inverses: &[String],
        public_cols: &[(String, usize)],
    );

    fn create_verifier_hpp(&mut self, name: &str, public_cols: &[(String, usize)]);
}

impl VerifierBuilder for BBFiles {
    fn create_verifier_cpp(
        &mut self,
        name: &str,
        witness: &[String],
        inverses: &[String],
        public_cols: &[(String, usize)],
    ) {
        let include_str = includes_cpp(&snake_case(name));

        let wire_transformation = |n: &String| {
            format!(
            "commitments.{n} = transcript->template receive_from_prover<Commitment>(commitment_labels.{n});"
        )
        };
        let wire_commitments = map_with_newline(witness, wire_transformation);

        let has_public_input_columns = !public_cols.is_empty();
        let has_inverses = !inverses.is_empty();

        let get_inverse_challenges = if has_inverses {
            "
            auto [beta, gamm] = transcript->template get_challenges<FF>(\"beta\", \"gamma\");
            relation_parameters.beta = beta;
            relation_parameters.gamma = gamm;
            "
            .to_string()
        } else {
            "".to_owned()
        };

        let verify_proof_function_declaration: String = if has_public_input_columns {
            format!("bool {name}Verifier::verify_proof(const HonkProof& proof, const std::vector<std::vector<FF>>& public_inputs)")
        } else {
            format!("bool {name}Verifier::verify_proof(const HonkProof& proof)")
        };

        let public_inputs_column_transformation =
            |public_inputs_column_name: &String, idx: usize| {
                format!(
                "
        FF {public_inputs_column_name}_evaluation = evaluate_public_input_column(public_inputs[{idx}], circuit_size, multivariate_challenge);
        if ({public_inputs_column_name}_evaluation != claimed_evaluations.{public_inputs_column_name}) {{
            return false;
        }}
                "
            )
            };

        let (public_inputs_check, evaluate_public_inputs) = if has_public_input_columns {
            let inputs_check = public_cols
                .iter()
                .map(|(col_name, idx)| public_inputs_column_transformation(col_name, *idx))
                .collect::<String>();

            let evaluate_public_inputs = format!(
                "

    using FF = {name}Flavor::FF;
    
    // Evaluate the given public input column over the multivariate challenge points
    [[maybe_unused]] inline FF evaluate_public_input_column(const std::vector<FF>& points, const size_t circuit_size, std::vector<FF> challenges) {{
        
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6361): we pad the points to the circuit size in order to get the correct evaluation.
        // This is not efficient, and will not be valid in production.
        std::vector<FF> new_points(circuit_size, 0);
        std::copy(points.begin(), points.end(), new_points.data());

        Polynomial<FF> polynomial(new_points);
        return polynomial.evaluate_mle(challenges);
    }}
                "
            );

            (inputs_check, evaluate_public_inputs)
        } else {
            ("".to_owned(), "".to_owned())
        };

        let inverse_commitments = map_with_newline(inverses, wire_transformation);

        let ver_cpp = format!("
{include_str} 

    namespace bb {{


    {name}Verifier::{name}Verifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
        : key(verifier_key)
    {{}}
    
    {name}Verifier::{name}Verifier({name}Verifier&& other) noexcept
        : key(std::move(other.key))
        , pcs_verification_key(std::move(other.pcs_verification_key))
    {{}}
    
    {name}Verifier& {name}Verifier::operator=({name}Verifier&& other) noexcept
    {{
        key = other.key;
        pcs_verification_key = (std::move(other.pcs_verification_key));
        commitments.clear();
        return *this;
    }}

    {evaluate_public_inputs}

    
    /**
     * @brief This function verifies an {name} Honk proof for given program settings.
     *
     */
    {verify_proof_function_declaration}
    {{
        using Flavor = {name}Flavor;
        using FF = Flavor::FF;
        using Commitment = Flavor::Commitment;
        // using PCS = Flavor::PCS;
        // using ZeroMorph = ZeroMorphVerifier_<PCS>;
        using VerifierCommitments = Flavor::VerifierCommitments;
        using CommitmentLabels = Flavor::CommitmentLabels;
    
        RelationParameters<FF> relation_parameters;
    
        transcript = std::make_shared<Transcript>(proof);
    
        VerifierCommitments commitments {{ key }};
        CommitmentLabels commitment_labels;
    
        const auto circuit_size = transcript->template receive_from_prover<uint32_t>(\"circuit_size\");
    
        if (circuit_size != key->circuit_size) {{
            return false;
        }}
    
        // Get commitments to VM wires
        {wire_commitments}

        {get_inverse_challenges}

        // Get commitments to inverses
        {inverse_commitments}
    
        // Execute Sumcheck Verifier
        const size_t log_circuit_size = numeric::get_msb(circuit_size);
        auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

        FF alpha = transcript->template get_challenge<FF>(\"Sumcheck:alpha\");

        auto gate_challenges = std::vector<FF>(log_circuit_size);
        for (size_t idx = 0; idx < log_circuit_size; idx++) {{
            gate_challenges[idx] = transcript->template get_challenge<FF>(\"Sumcheck:gate_challenge_\" + std::to_string(idx));
        }}

        auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
            sumcheck.verify(relation_parameters, alpha, gate_challenges);
    
        // If Sumcheck did not verify, return false
        if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {{
            return false;
        }}

        // Public columns evaluation checks
        {public_inputs_check}
    
        // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
        // unrolled protocol.
        // NOTE: temporarily disabled - facing integration issues
        // auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
        //                                         commitments.get_to_be_shifted(),
        //                                         claimed_evaluations.get_unshifted(),
        //                                         claimed_evaluations.get_shifted(),
        //                                         multivariate_challenge,
        //                                         transcript);
    
        // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
        // return sumcheck_verified.value() && verified;
        return sumcheck_verified.value();
    }}
    
    
    }} // namespace bb
    
    
    ");

        self.write_file(
            &self.prover,
            &format!("{}_verifier.cpp", snake_case(name)),
            &ver_cpp,
        );
    }

    fn create_verifier_hpp(&mut self, name: &str, public_cols: &[(String, usize)]) {
        let include_str = include_hpp(&snake_case(name));

        // If there are public input columns, then the generated verifier must take them in as an argument for the verify_proof
        let verify_proof = if !public_cols.is_empty() {
            "bool verify_proof(const HonkProof& proof, const std::vector<std::vector<FF>>& public_inputs);"
                .to_string()
        } else {
            "bool verify_proof(const HonkProof& proof);".to_owned()
        };

        let ver_hpp = format!(
            "
{include_str}
    
    namespace bb {{
    class {name}Verifier {{
        using Flavor = {name}Flavor;
        using FF = Flavor::FF;
        using Commitment = Flavor::Commitment;
        using VerificationKey = Flavor::VerificationKey;
        using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;
        using Transcript = Flavor::Transcript;
    
    public:
        explicit {name}Verifier(std::shared_ptr<VerificationKey> verifier_key = nullptr);
        {name}Verifier({name}Verifier&& other) noexcept;
        {name}Verifier(const {name}Verifier& other) = delete;
    
        {name}Verifier& operator=(const {name}Verifier& other) = delete;
        {name}Verifier& operator=({name}Verifier&& other) noexcept;
    
        {verify_proof}
    
        std::shared_ptr<VerificationKey> key;
        std::map<std::string, Commitment> commitments;
        std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
        std::shared_ptr<Transcript> transcript;
    }};
    
    }} // namespace bb
     
    
    "
        );

        self.write_file(
            &self.prover,
            &format!("{}_verifier.hpp", snake_case(name)),
            &ver_hpp,
        );
    }
}

fn include_hpp(name: &str) -> String {
    format!(
        "
#pragma once
#include \"barretenberg/plonk/proof_system/types/proof.hpp\"
#include \"barretenberg/sumcheck/sumcheck.hpp\"
#include \"barretenberg/vm/generated/{name}_flavor.hpp\"
#include \"barretenberg/vm/avm_trace/constants.hpp\"
"
    )
}

fn includes_cpp(name: &str) -> String {
    format!(
        "
    #include \"./{name}_verifier.hpp\"
    #include \"barretenberg/commitment_schemes/zeromorph/zeromorph.hpp\"
    #include \"barretenberg/numeric/bitop/get_msb.hpp\"
    #include \"barretenberg/polynomials/polynomial.hpp\"
    #include \"barretenberg/transcript/transcript.hpp\"
    "
    )
}
