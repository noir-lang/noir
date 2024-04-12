import { PROVING_STATUS, makeEmptyProcessedTx } from '@aztec/circuit-types';
import { Fr, type GlobalVariables, Header } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import * as fs from 'fs/promises';
import { type MemDown, default as memdown } from 'memdown';

import { getConfig, makeBloatedProcessedTx, makeGlobals } from '../mocks/fixtures.js';
import { buildBaseRollupInput } from '../orchestrator/block-building-helpers.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { BBNativeRollupProver, type BBProverConfig } from './bb_prover.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:bb-prover-test');

describe('prover/bb_prover', () => {
  let builderDb: MerkleTreeOperations;
  let prover: BBNativeRollupProver;
  let directoryToCleanup: string | undefined;

  let blockNumber: number;

  let globalVariables: GlobalVariables;

  beforeAll(async () => {
    const config = await getConfig(logger);
    if (!config) {
      throw new Error(`BB and ACVM binaries must be present to test the BB Prover`);
    }
    directoryToCleanup = config.directoryToCleanup;
    const bbConfig: BBProverConfig = {
      acvmBinaryPath: config.expectedAcvmPath,
      acvmWorkingDirectory: config.acvmWorkingDirectory,
      bbBinaryPath: config.expectedBBPath,
      bbWorkingDirectory: config.bbWorkingDirectory,
    };
    prover = await BBNativeRollupProver.new(bbConfig);
  }, 60_000);

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = makeGlobals(blockNumber);

    builderDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
  }, 60_000);

  afterAll(async () => {
    if (directoryToCleanup) {
      await fs.rm(directoryToCleanup, { recursive: true, force: true });
    }
  }, 5000);

  it('proves the base rollup', async () => {
    const txs = await Promise.all([makeBloatedProcessedTx(builderDb, 1)]);

    logger.verbose('Building base rollup inputs');
    const baseRollupInputs = [];
    for (const tx of txs) {
      baseRollupInputs.push(await buildBaseRollupInput(tx, globalVariables, builderDb));
    }
    logger.verbose('Proving base rollups');
    const proofOutputs = await Promise.all(baseRollupInputs.map(inputs => prover.getBaseRollupProof(inputs)));
    logger.verbose('Verifying base rollups');
    await expect(
      Promise.all(proofOutputs.map(output => prover.verifyProof('BaseRollupArtifact', output[1]))),
    ).resolves.not.toThrow();
  }, 600_000);

  it('proves all circuits', async () => {
    const txs = await Promise.all([
      makeBloatedProcessedTx(builderDb, 1),
      makeBloatedProcessedTx(builderDb, 2),
      makeBloatedProcessedTx(builderDb, 3),
      makeBloatedProcessedTx(builderDb, 4),
    ]);

    const orchestrator = await ProvingOrchestrator.new(builderDb, prover);

    const provingTicket = await orchestrator.startNewBlock(
      4,
      globalVariables,
      [],
      makeEmptyProcessedTx(Header.empty(), new Fr(1234), new Fr(1)),
    );

    for (const tx of txs) {
      await orchestrator.addNewTx(tx);
    }

    await orchestrator.setBlockCompleted();

    const provingResult = await provingTicket.provingPromise;

    expect(provingResult.status).toBe(PROVING_STATUS.SUCCESS);

    const blockResult = await orchestrator.finaliseBlock();

    await expect(prover.verifyProof('RootRollupArtifact', blockResult.proof)).resolves.not.toThrow();

    await orchestrator.stop();
  }, 600_000);
});
