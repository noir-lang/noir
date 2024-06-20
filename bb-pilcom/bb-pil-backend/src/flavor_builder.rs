use crate::{
    file_writer::BBFiles,
    utils::{get_relations_imports, map_with_newline, snake_case},
};

pub trait FlavorBuilder {
    #[allow(clippy::too_many_arguments)]
    fn create_flavor_hpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        lookups: &[String],
        fixed: &[String],
        witness: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    );
}

/// Build the boilerplate for the flavor file
impl FlavorBuilder for BBFiles {
    fn create_flavor_hpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        lookups: &[String],
        fixed: &[String],
        witness: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    ) {
        let first_poly = &witness[0];
        let includes = flavor_includes(&snake_case(name), relation_file_names, lookups);
        let num_precomputed = fixed.len();
        let num_witness = witness.len();
        let num_all = all_cols_and_shifts.len();

        // Top of file boilerplate
        let class_aliases = create_class_aliases();
        let relation_definitions = create_relation_definitions(name, relation_file_names, lookups);
        let container_size_definitions =
            container_size_definitions(num_precomputed, num_witness, num_all);

        // Entities classes
        let precomputed_entities = create_precomputed_entities(fixed);
        let witness_entities = create_witness_entities(witness);
        let all_entities =
            create_all_entities(all_cols, to_be_shifted, shifted, all_cols_and_shifts);

        let proving_and_verification_key =
            create_proving_and_verification_key(name, lookups, to_be_shifted);
        let polynomial_views = create_polynomial_views(first_poly);

        let commitment_labels_class = create_commitment_labels(all_cols);

        let verification_commitments = create_verifier_commitments(fixed);

        let transcript = generate_transcript(witness);

        let flavor_hpp = format!(
            "
{includes}

namespace bb {{

class {name}Flavor {{
    public: 
        {class_aliases}

        {container_size_definitions}

        {relation_definitions}

        static constexpr bool has_zero_row = true;

    private:
        {precomputed_entities} 

        {witness_entities}

        {all_entities}

    
        {proving_and_verification_key}


        {polynomial_views}

    {commitment_labels_class}

    {verification_commitments}

    {transcript}
}};

}} // namespace bb
    
    
    "
        );

        self.write_file(
            &self.flavor,
            &format!("{}_flavor.hpp", snake_case(name)),
            &flavor_hpp,
        );
    }
}

/// Imports located at the top of the flavor files
fn flavor_includes(name: &str, relation_file_names: &[String], lookups: &[String]) -> String {
    let relation_imports = get_relations_imports(name, relation_file_names, lookups);

    format!(
        "#pragma once

#include \"barretenberg/commitment_schemes/kzg/kzg.hpp\"
#include \"barretenberg/ecc/curves/bn254/g1.hpp\"
#include \"barretenberg/flavor/relation_definitions.hpp\"
#include \"barretenberg/polynomials/barycentric.hpp\"
#include \"barretenberg/polynomials/univariate.hpp\"

#include \"barretenberg/relations/generic_permutation/generic_permutation_relation.hpp\"

#include \"barretenberg/flavor/flavor_macros.hpp\"
#include \"barretenberg/transcript/transcript.hpp\"
#include \"barretenberg/polynomials/evaluation_domain.hpp\"
#include \"barretenberg/polynomials/polynomial.hpp\"
#include \"barretenberg/flavor/flavor.hpp\"
{relation_imports}
"
    )
}

/// Creates comma separated relations tuple file
fn create_relations_tuple(master_name: &str, relation_file_names: &[String]) -> String {
    relation_file_names
        .iter()
        .map(|name| format!("{master_name}_vm::{name}<FF>"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Creates comma separated relations tuple file
fn create_lookups_tuple(lookups: &[String]) -> Option<String> {
    if lookups.is_empty() {
        return None;
    }
    Some(
        lookups
            .iter()
            .map(|lookup| format!("{}_relation<FF>", lookup.clone()))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

/// Create Class Aliases
///
/// Contains boilerplate defining key characteristics of the flavor class
fn create_class_aliases() -> &'static str {
    r#"
        using Curve = curve::BN254;
        using G1 = Curve::Group;
        using PCS = KZG<Curve>;

        using FF = G1::subgroup_field;
        using Polynomial = bb::Polynomial<FF>;
        using PolynomialHandle = std::span<FF>;
        using GroupElement = G1::element;
        using Commitment = G1::affine_element;
        using CommitmentHandle = G1::affine_element;
        using CommitmentKey = bb::CommitmentKey<Curve>;
        using VerifierCommitmentKey = bb::VerifierCommitmentKey<Curve>;
        using RelationSeparator = FF;
    "#
}

/// Create relation definitions
///
/// Contains all of the boilerplate code required to generate relation definitions.
/// We instantiate the Relations container, which contains a tuple of all of the separate relation file
/// definitions.
///
/// We then also define some constants, making use of the preprocessor.
fn create_relation_definitions(
    name: &str,
    relation_file_names: &[String],
    lookups: &[String],
) -> String {
    // Relations tuple = ns::relation_name_0, ns::relation_name_1, ... ns::relation_name_n (comma speratated)
    let comma_sep_relations = create_relations_tuple(name, relation_file_names);
    let comma_sep_lookups: Option<String> = create_lookups_tuple(lookups);

    // We only include the grand product relations if we are given lookups
    let mut grand_product_relations = String::new();
    let mut all_relations = comma_sep_relations.to_string();
    if let Some(lookups) = comma_sep_lookups {
        all_relations = all_relations + &format!(", {lookups}");
        grand_product_relations = format!("using GrandProductRelations = std::tuple<{lookups}>;");
    }

    format!("
        {grand_product_relations}

        using Relations = std::tuple<{all_relations}>;

        static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

        // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
        // random polynomial e.g. For \\sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
        // length = 3
        static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
        static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

        template <size_t NUM_INSTANCES>
        using ProtogalaxyTupleOfTuplesOfUnivariates =
            decltype(create_protogalaxy_tuple_of_tuples_of_univariates<Relations, NUM_INSTANCES>());
        using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
        using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());
        ")
}

/// Create the number of columns boilerplate for the flavor file
fn container_size_definitions(
    num_precomputed: usize,
    num_witness: usize,
    num_all: usize,
) -> String {
    format!("
        static constexpr size_t NUM_PRECOMPUTED_ENTITIES = {num_precomputed}; 
        static constexpr size_t NUM_WITNESS_ENTITIES = {num_witness};
        static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
        // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for the unshifted and one for the shifted
        static constexpr size_t NUM_ALL_ENTITIES = {num_all};

    ")
}

/// Returns a Ref Vector with the given name,
///
/// The vector returned will reference the columns names given
/// Used in all entities declarations
fn return_ref_vector(name: &str, columns: &[String]) -> String {
    let comma_sep = create_comma_separated(columns);

    format!("RefVector<DataType> {name}() {{ return {{ {comma_sep} }}; }};")
}

/// list -> "list[0], list[1], ... list[n-1]"
fn create_comma_separated(list: &[String]) -> String {
    list.join(", ")
}

/// Create Precomputed Entities
///
/// Precomputed first contains a pointer view defining all of the precomputed columns
/// As-well as any polys conforming to tables / ids / permutations
fn create_precomputed_entities(fixed: &[String]) -> String {
    let pointer_view = create_flavor_members(fixed);

    let selectors = return_ref_vector("get_selectors", fixed);
    let sigma_polys = return_ref_vector("get_sigma_polynomials", &[]);
    let id_polys = return_ref_vector("get_id_polynomials", &[]);
    let table_polys = return_ref_vector("get_table_polynomials", &[]);

    format!(
        "
        template<typename DataType_>
        class PrecomputedEntities : public PrecomputedEntitiesBase {{
            public:
              using DataType = DataType_;

              {pointer_view}

              {selectors}
              {sigma_polys}
              {id_polys}
              {table_polys}
          }};
        "
    )
}

fn create_witness_entities(witness: &[String]) -> String {
    let pointer_view = create_flavor_members(witness);

    let wires = return_ref_vector("get_wires", witness);

    format!(
        "
        template <typename DataType>
        class WitnessEntities {{
            public:

            {pointer_view}

            {wires} 
        }};
        "
    )
}

/// Creates container of all witness entities and shifts
fn create_all_entities(
    all_cols: &[String],
    to_be_shifted: &[String],
    shifted: &[String],
    all_cols_and_shifts: &[String],
) -> String {
    let all_entities_flavor_members = create_flavor_members(all_cols_and_shifts);

    let wires = return_ref_vector("get_wires", all_cols_and_shifts);
    let get_unshifted = return_ref_vector("get_unshifted", all_cols);
    let get_to_be_shifted = return_ref_vector("get_to_be_shifted", to_be_shifted);
    let get_shifted = return_ref_vector("get_shifted", shifted);

    format!(
        "
        template <typename DataType>
        class AllEntities {{
            public:

            {all_entities_flavor_members}


            {wires}
            {get_unshifted}
            {get_to_be_shifted}
            {get_shifted}
        }};
        "
    )
}

fn create_proving_and_verification_key(
    flavor_name: &str,
    lookups: &[String],
    to_be_shifted: &[String],
) -> String {
    let get_to_be_shifted = return_ref_vector("get_to_be_shifted", to_be_shifted);
    let compute_logderivative_inverses =
        create_compute_logderivative_inverses(flavor_name, lookups);

    format!("
        public:
        class ProvingKey : public ProvingKeyAvm_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey> {{
            public:
            // Expose constructors on the base class
            using Base = ProvingKeyAvm_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey>;
            using Base::Base;

            {get_to_be_shifted}

            {compute_logderivative_inverses}
        }};

        using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey>;
    ")
}

fn create_polynomial_views(first_poly: &String) -> String {
    format!("

    class AllValues : public AllEntities<FF> {{
        public:
          using Base = AllEntities<FF>;
          using Base::Base;
      }};
  
    /**
     * @brief A container for the prover polynomials handles.
    */
    class ProverPolynomials : public AllEntities<Polynomial> {{
      public:
        // Define all operations as default, except copy construction/assignment
        ProverPolynomials() = default;
        ProverPolynomials& operator=(const ProverPolynomials&) = delete;
        ProverPolynomials(const ProverPolynomials& o) = delete;
        ProverPolynomials(ProverPolynomials&& o) noexcept = default;
        ProverPolynomials& operator=(ProverPolynomials&& o) noexcept = default;
        ~ProverPolynomials() = default;
        
        ProverPolynomials(ProvingKey& proving_key)
        {{
            for (auto [prover_poly, key_poly] : zip_view(this->get_unshifted(), proving_key.get_all())) {{
                ASSERT(flavor_get_label(*this, prover_poly) == flavor_get_label(proving_key, key_poly));
                prover_poly = key_poly.share();
            }}
            for (auto [prover_poly, key_poly] : zip_view(this->get_shifted(), proving_key.get_to_be_shifted())) {{
                ASSERT(flavor_get_label(*this, prover_poly) == (flavor_get_label(proving_key, key_poly) + \"_shift\"));
                prover_poly = key_poly.shifted();
            }}
        }}

        [[nodiscard]] size_t get_polynomial_size() const {{ return {first_poly}.size(); }}
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
        * represents one row in the execution trace.
        */
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {{
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {{
                result_field = polynomial[row_idx];
            }}
          return result;
        }}
    }};

    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial> {{
      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {{
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : get_all()) {{
                poly = Polynomial(circuit_size / 2);
            }}
        }}
    }};

    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck.
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH>
    using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck with some of the computation
     * optimistically ignored
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH, size_t SKIP_COUNT>
    using OptimisedProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH, 0, SKIP_COUNT>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the witness commitments.
     *
     */
    using WitnessCommitments = WitnessEntities<Commitment>;

    ")
}

fn create_flavor_members(entities: &[String]) -> String {
    let pointer_list = create_comma_separated(entities);

    format!(
        "DEFINE_FLAVOR_MEMBERS(DataType, {pointer_list})",
        pointer_list = pointer_list
    )
}

fn create_labels(all_ents: &[String]) -> String {
    let mut labels = String::new();
    for name in all_ents {
        labels.push_str(&format!(
            "Base::{name} = \"{}\"; 
            ",
            name.to_uppercase()
        ));
    }
    labels
}

fn create_commitment_labels(all_ents: &[String]) -> String {
    let labels = create_labels(all_ents);

    format!(
        "
        class CommitmentLabels: public AllEntities<std::string> {{
            private:
                using Base = AllEntities<std::string>;

            public:
                CommitmentLabels() : AllEntities<std::string>()
            {{
                {labels}
            }};
        }};
        "
    )
}

/// Create the compute_logderivative_inverses function
///
/// If we do not have any lookups, we do not need to include this round
fn create_compute_logderivative_inverses(flavor_name: &str, lookups: &[String]) -> String {
    if lookups.is_empty() {
        return "".to_string();
    }

    let compute_inverse_transformation = |lookup_name: &String| {
        format!("bb::compute_logderivative_inverse<{flavor_name}Flavor, {lookup_name}_relation<FF>>(prover_polynomials, relation_parameters, this->circuit_size);")
    };

    let compute_inverses = map_with_newline(lookups, compute_inverse_transformation);

    format!(
        "
        void compute_logderivative_inverses(const RelationParameters<FF>& relation_parameters)
        {{
            ProverPolynomials prover_polynomials = ProverPolynomials(*this);

            {compute_inverses}
        }}
        "
    )
}

fn create_key_dereference(fixed: &[String]) -> String {
    let deref_transformation = |name: &String| format!("{name} = verification_key->{name};");

    map_with_newline(fixed, deref_transformation)
}

fn create_verifier_commitments(fixed: &[String]) -> String {
    let key_dereference = create_key_dereference(fixed);

    format!(
        "
    class VerifierCommitments : public AllEntities<Commitment> {{
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {{
            {key_dereference}
        }}
    }};
"
    )
}

fn generate_transcript(witness: &[String]) -> String {
    // Transformations
    let declaration_transform = |c: &_| format!("Commitment {c};");
    let deserialize_transform = |name: &_| {
        format!(
            "{name} = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_frs_read);",
        )
    };
    let serialize_transform =
        |name: &_| format!("serialize_to_buffer<Commitment>({name}, Transcript::proof_data);");

    // Perform Transformations
    let declarations = map_with_newline(witness, declaration_transform);
    let deserialize_wires = map_with_newline(witness, deserialize_transform);
    let serialize_wires = map_with_newline(witness, serialize_transform);

    format!("
    class Transcript : public NativeTranscript {{
      public:
        uint32_t circuit_size;

        {declarations}

        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        Transcript(const std::vector<FF>& proof)
            : NativeTranscript(proof)
        {{}}

        void deserialize_full_transcript()
        {{
            size_t num_frs_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_frs_read);
            size_t log_n = numeric::get_msb(circuit_size);

            {deserialize_wires}

            for (size_t i = 0; i < log_n; ++i) {{
                sumcheck_univariates.emplace_back(
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                        Transcript::proof_data, num_frs_read));
            }}
            sumcheck_evaluations = deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(
                Transcript::proof_data, num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {{
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_frs_read));
            }}
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
        }}

        void serialize_full_transcript()
        {{
            size_t old_proof_length = proof_data.size();
            Transcript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            serialize_to_buffer(circuit_size, Transcript::proof_data);

            {serialize_wires}

            for (size_t i = 0; i < log_n; ++i) {{
                serialize_to_buffer(sumcheck_univariates[i], Transcript::proof_data);
            }}
            serialize_to_buffer(sumcheck_evaluations, Transcript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {{
                serialize_to_buffer(zm_cq_comms[i], proof_data);
            }}
            serialize_to_buffer(zm_cq_comm, proof_data);
            serialize_to_buffer(zm_pi_comm, proof_data);

            // sanity check to make sure we generate the same length of proof as before.
            ASSERT(proof_data.size() == old_proof_length);
        }}
    }};
    ")
}
