import { createDebugLogger } from '@aztec/foundation/log';

import { makeBloatedProcessedTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';
import { buildBaseRollupInput } from '../orchestrator/block-building-helpers.js';
import { BBNativeRollupProver, type BBProverConfig } from './bb_prover.js';

const logger = createDebugLogger('aztec:bb-prover-base-rollup');

describe('prover/bb_prover/base-rollup', () => {
  let context: TestContext;

  beforeAll(async () => {
    const buildProver = (bbConfig: BBProverConfig) => {
      bbConfig.circuitFilter = ['BaseRollupArtifact'];
      return BBNativeRollupProver.new(bbConfig);
    };
    context = await TestContext.new(logger, 1, buildProver);
  }, 60_000);

  afterAll(async () => {
    await context.cleanup();
  }, 5000);

  it('proves the base rollup', async () => {
    const tx = await makeBloatedProcessedTx(context.actualDb, 1);

    logger.verbose('Building base rollup inputs');
    const baseRollupInputs = await buildBaseRollupInput(tx, context.globalVariables, context.actualDb);
    logger.verbose('Proving base rollups');
    const proofOutputs = await context.prover.getBaseRollupProof(baseRollupInputs);
    logger.verbose('Verifying base rollups');
    await expect(context.prover.verifyProof('BaseRollupArtifact', proofOutputs.proof)).resolves.not.toThrow();
  }, 200_000);
});
