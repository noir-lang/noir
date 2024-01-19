#include "join_split.hpp"
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

using namespace bb::plonk;
using namespace bb::plonk::stdlib::merkle_tree;

static std::shared_ptr<plonk::proving_key> proving_key;
static std::shared_ptr<plonk::verification_key> verification_key;

void init_proving_key(bool mock)
{
    if (proving_key) {
        return;
    }

    // Junk data required just to create proving key.
    join_split_tx tx = noop_tx();

    if (!mock) {
        Builder builder;
        join_split_circuit(builder, tx);
        Composer composer;
        proving_key = composer.compute_proving_key(builder);
    } else {
        Builder builder;
        join_split_circuit(builder, tx);
        Composer composer;
        join_split_example::proofs::mock::mock_circuit(builder, builder.get_public_inputs());
        proving_key = composer.compute_proving_key(builder);
    }
}

void release_proving_key()
{
    proving_key.reset();
}

void init_verification_key()
{
    if (!proving_key) {
        std::abort();
    }

    verification_key =
        bb::plonk::compute_verification_key_common(proving_key, srs::get_crs_factory()->get_verifier_crs());
}

Prover new_join_split_prover(join_split_tx const& tx, bool mock)
{
    Builder builder;
    join_split_circuit(builder, tx);

    if (builder.failed()) {
        std::string error = format("builder logic failed: ", builder.err());
        throw_or_abort(error);
    }

    info("public inputs: ", builder.public_inputs.size());

    Composer composer(proving_key, nullptr);
    if (!mock) {
        info("composer gates: ", builder.get_num_gates());
        return composer.create_prover(builder);
    } else {
        Composer mock_proof_composer(proving_key, nullptr);
        join_split_example::proofs::mock::mock_circuit(builder, builder.get_public_inputs());
        info("mock composer gates: ", builder.get_num_gates());
        return mock_proof_composer.create_prover(builder);
    }
}

bool verify_proof(plonk::proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(verification_key->num_public_inputs));

    std::unique_ptr<plonk::KateCommitmentScheme<plonk::ultra_settings>> kate_commitment_scheme =
        std::make_unique<plonk::KateCommitmentScheme<plonk::ultra_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    return verifier.verify_proof(proof);
}

std::shared_ptr<plonk::proving_key> get_proving_key()
{
    return proving_key;
}

std::shared_ptr<plonk::verification_key> get_verification_key()
{
    return verification_key;
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
