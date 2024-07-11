import { BBNativeRollupProver, type BBProverConfig } from '@aztec/bb-prover';
import { makePaddingProcessedTxFromTubeProof } from '@aztec/circuit-types';
import { NESTED_RECURSIVE_PROOF_LENGTH, makeEmptyRecursiveProof } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { TestContext } from '../mocks/test_context.js';
import { buildBaseRollupInput } from '../orchestrator/block-building-helpers.js';

const logger = createDebugLogger('aztec:bb-prover-base-rollup');

describe('prover/bb_prover/base-rollup', () => {
  let context: TestContext;
  let prover: BBNativeRollupProver;

  beforeAll(async () => {
    const buildProver = async (bbConfig: BBProverConfig) => {
      prover = await BBNativeRollupProver.new(bbConfig, new NoopTelemetryClient());
      return prover;
    };
    context = await TestContext.new(logger, 1, buildProver);
  });

  afterAll(async () => {
    await context.cleanup();
  });

  it('proves the base rollup', async () => {
    const header = await context.actualDb.buildInitialHeader();
    const chainId = context.globalVariables.chainId;
    const version = context.globalVariables.version;
    const vkTreeRoot = getVKTreeRoot();

    const inputs = {
      header,
      chainId,
      version,
      vkTreeRoot,
    };

    const paddingTxPublicInputsAndProof = await context.prover.getEmptyTubeProof(inputs);
    const tx = makePaddingProcessedTxFromTubeProof(paddingTxPublicInputsAndProof);

    logger.verbose('Building base rollup inputs');
    const baseRollupInputs = await buildBaseRollupInput(
      tx,
      makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      context.globalVariables,
      context.actualDb,
      paddingTxPublicInputsAndProof.verificationKey,
    );
    logger.verbose('Proving base rollups');
    const proofOutputs = await context.prover.getBaseRollupProof(baseRollupInputs);
    logger.verbose('Verifying base rollups');
    await expect(prover.verifyProof('BaseRollupArtifact', proofOutputs.proof.binaryProof)).resolves.not.toThrow();
  });
});
