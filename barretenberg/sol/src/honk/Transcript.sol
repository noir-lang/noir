import {
    Honk,
    NUMBER_OF_ALPHAS,
    NUMBER_OF_ENTITIES,
    BATCHED_RELATION_PARTIAL_LENGTH,
    CONST_PROOF_SIZE_LOG_N
} from "./HonkTypes.sol";
import {Fr, FrLib} from "./Fr.sol";
import {LOG_N, NUMBER_OF_PUBLIC_INPUTS} from "./keys/Add2HonkVerificationKey.sol";

// Transcript library to generate fiat shamir challenges
struct Transcript {
    Fr eta;
    Fr etaTwo;
    Fr etaThree;
    Fr beta;
    Fr gamma;
    Fr[NUMBER_OF_ALPHAS] alphas;
    Fr[LOG_N] gateChallenges;
    Fr[CONST_PROOF_SIZE_LOG_N] sumCheckUChallenges;
    Fr rho;
    // Zero morph
    Fr zmX;
    Fr zmY;
    Fr zmZ;
    Fr zmQuotient;
    // Derived
    Fr publicInputsDelta;
}

library TranscriptLib {
    function generateTranscript(
        Honk.Proof memory proof,
        Honk.VerificationKey memory vk,
        bytes32[] calldata publicInputs
    ) internal view returns (Transcript memory t) {
        (t.eta, t.etaTwo, t.etaThree) = generateEtaChallenge(proof, publicInputs);

        (t.beta, t.gamma) = generateBetaAndGammaChallenges(t.etaThree, proof);

        t.alphas = generateAlphaChallenges(t.gamma, proof);

        t.gateChallenges = generateGateChallenges(t.alphas[NUMBER_OF_ALPHAS - 1]);

        t.sumCheckUChallenges = generateSumcheckChallenges(proof, t.gateChallenges[LOG_N - 1]);
        t.rho = generateRhoChallenge(proof, t.sumCheckUChallenges[CONST_PROOF_SIZE_LOG_N - 1]);

        t.zmY = generateZMYChallenge(t.rho, proof);

        (t.zmX, t.zmZ) = generateZMXZChallenges(t.zmY, proof);

        return t;
    }

    function generateEtaChallenge(Honk.Proof memory proof, bytes32[] calldata publicInputs)
        internal
        view
        returns (Fr eta, Fr etaTwo, Fr etaThree)
    {
        bytes32[3 + NUMBER_OF_PUBLIC_INPUTS + 12] memory round0;
        round0[0] = bytes32(proof.circuitSize);
        round0[1] = bytes32(proof.publicInputsSize);
        round0[2] = bytes32(proof.publicInputsOffset);
        for (uint256 i = 0; i < NUMBER_OF_PUBLIC_INPUTS; i++) {
            round0[3 + i] = bytes32(publicInputs[i]);
        }

        // Create the first challenge
        // Note: w4 is added to the challenge later on
        round0[3 + NUMBER_OF_PUBLIC_INPUTS] = bytes32(proof.w1.x_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 1] = bytes32(proof.w1.x_1);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 2] = bytes32(proof.w1.y_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 3] = bytes32(proof.w1.y_1);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 4] = bytes32(proof.w2.x_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 5] = bytes32(proof.w2.x_1);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 6] = bytes32(proof.w2.y_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 7] = bytes32(proof.w2.y_1);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 8] = bytes32(proof.w3.x_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 9] = bytes32(proof.w3.x_1);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 10] = bytes32(proof.w3.y_0);
        round0[3 + NUMBER_OF_PUBLIC_INPUTS + 11] = bytes32(proof.w3.y_1);

        eta = FrLib.fromBytes32(keccak256(abi.encodePacked(round0)));
        etaTwo = FrLib.fromBytes32(keccak256(abi.encodePacked(Fr.unwrap(eta))));
        etaThree = FrLib.fromBytes32(keccak256(abi.encodePacked(Fr.unwrap(etaTwo))));
    }

    function generateBetaAndGammaChallenges(Fr previousChallenge, Honk.Proof memory proof)
        internal
        view
        returns (Fr beta, Fr gamma)
    {
        bytes32[13] memory round1;
        round1[0] = FrLib.toBytes32(previousChallenge);
        round1[1] = bytes32(proof.lookupReadCounts.x_0);
        round1[2] = bytes32(proof.lookupReadCounts.x_1);
        round1[3] = bytes32(proof.lookupReadCounts.y_0);
        round1[4] = bytes32(proof.lookupReadCounts.y_1);
        round1[5] = bytes32(proof.lookupReadTags.x_0);
        round1[6] = bytes32(proof.lookupReadTags.x_1);
        round1[7] = bytes32(proof.lookupReadTags.y_0);
        round1[8] = bytes32(proof.lookupReadTags.y_1);
        round1[9] = bytes32(proof.w4.x_0);
        round1[10] = bytes32(proof.w4.x_1);
        round1[11] = bytes32(proof.w4.y_0);
        round1[12] = bytes32(proof.w4.y_1);

        beta = FrLib.fromBytes32(keccak256(abi.encodePacked(round1)));
        gamma = FrLib.fromBytes32(keccak256(abi.encodePacked(beta)));
    }

    // Alpha challenges non-linearise the gate contributions
    function generateAlphaChallenges(Fr previousChallenge, Honk.Proof memory proof)
        internal
        view
        returns (Fr[NUMBER_OF_ALPHAS] memory alphas)
    {
        // Generate the original sumcheck alpha 0 by hashing zPerm and zLookup
        uint256[9] memory alpha0;
        alpha0[0] = Fr.unwrap(previousChallenge);
        alpha0[1] = proof.lookupInverses.x_0;
        alpha0[2] = proof.lookupInverses.x_1;
        alpha0[3] = proof.lookupInverses.y_0;
        alpha0[4] = proof.lookupInverses.y_1;
        alpha0[5] = proof.zPerm.x_0;
        alpha0[6] = proof.zPerm.x_1;
        alpha0[7] = proof.zPerm.y_0;
        alpha0[8] = proof.zPerm.y_1;

        alphas[0] = FrLib.fromBytes32(keccak256(abi.encodePacked(alpha0)));

        Fr prevChallenge = alphas[0];
        for (uint256 i = 1; i < NUMBER_OF_ALPHAS; i++) {
            prevChallenge = FrLib.fromBytes32(keccak256(abi.encodePacked(Fr.unwrap(prevChallenge))));
            alphas[i] = prevChallenge;
        }
    }

    function generateGateChallenges(Fr previousChallenge) internal view returns (Fr[LOG_N] memory gateChallenges) {
        for (uint256 i = 0; i < LOG_N; i++) {
            previousChallenge = FrLib.fromBytes32(keccak256(abi.encodePacked(Fr.unwrap(previousChallenge))));
            gateChallenges[i] = previousChallenge;
        }
    }

    function generateSumcheckChallenges(Honk.Proof memory proof, Fr prevChallenge)
        internal
        view
        returns (Fr[CONST_PROOF_SIZE_LOG_N] memory sumcheckChallenges)
    {
        for (uint256 i = 0; i < CONST_PROOF_SIZE_LOG_N; i++) {
            Fr[BATCHED_RELATION_PARTIAL_LENGTH + 1] memory univariateChal;
            univariateChal[0] = prevChallenge;

            // TODO(opt): memcpy
            for (uint256 j = 0; j < BATCHED_RELATION_PARTIAL_LENGTH; j++) {
                univariateChal[j + 1] = proof.sumcheckUnivariates[i][j];
            }

            sumcheckChallenges[i] = FrLib.fromBytes32(keccak256(abi.encodePacked(univariateChal)));
            prevChallenge = sumcheckChallenges[i];
        }
    }

    function generateRhoChallenge(Honk.Proof memory proof, Fr prevChallenge) internal view returns (Fr rho) {
        Fr[NUMBER_OF_ENTITIES + 1] memory rhoChallengeElements;
        rhoChallengeElements[0] = prevChallenge;

        // TODO: memcpy
        for (uint256 i = 0; i < NUMBER_OF_ENTITIES; i++) {
            rhoChallengeElements[i + 1] = proof.sumcheckEvaluations[i];
        }

        rho = FrLib.fromBytes32(keccak256(abi.encodePacked(rhoChallengeElements)));
    }

    function generateZMYChallenge(Fr previousChallenge, Honk.Proof memory proof)
        internal
        view
        returns (Fr zeromorphY)
    {
        uint256[CONST_PROOF_SIZE_LOG_N * 4 + 1] memory zmY;
        zmY[0] = Fr.unwrap(previousChallenge);

        for (uint256 i; i < CONST_PROOF_SIZE_LOG_N; ++i) {
            zmY[1 + i * 4] = proof.zmCqs[i].x_0;
            zmY[2 + i * 4] = proof.zmCqs[i].x_1;
            zmY[3 + i * 4] = proof.zmCqs[i].y_0;
            zmY[4 + i * 4] = proof.zmCqs[i].y_1;
        }

        zeromorphY = FrLib.fromBytes32(keccak256(abi.encodePacked(zmY)));
    }

    function generateZMXZChallenges(Fr previousChallenge, Honk.Proof memory proof)
        internal
        view
        returns (Fr zeromorphX, Fr zeromorphZ)
    {
        uint256[4 + 1] memory buf;
        buf[0] = Fr.unwrap(previousChallenge);

        buf[1] = proof.zmCq.x_0;
        buf[2] = proof.zmCq.x_1;
        buf[3] = proof.zmCq.y_0;
        buf[4] = proof.zmCq.y_1;

        zeromorphX = FrLib.fromBytes32(keccak256(abi.encodePacked(buf)));
        zeromorphZ = FrLib.fromBytes32(keccak256(abi.encodePacked(zeromorphX)));
    }
}
